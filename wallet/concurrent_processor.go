package wallet

import (
	"context"
	"fmt"
	"sync"
	"sync/atomic"
	"time"

	"github.com/stellar/go/keypair"
)

// Concurrent processor for simultaneous claim and transfer operations
type ConcurrentProcessor struct {
	wallet         *Wallet
	claimAttempts  int64
	transferAttempts int64
	successCount   int64
	activeOps      sync.Map
	
	// Configuration
	maxConcurrentOps int
	retryInterval    time.Duration
}

func NewConcurrentProcessor(wallet *Wallet) *ConcurrentProcessor {
	return &ConcurrentProcessor{
		wallet:           wallet,
		maxConcurrentOps: 1000,
		retryInterval:    time.Millisecond,
	}
}

// Execute independent concurrent operations
func (cp *ConcurrentProcessor) ExecuteIndependentOperations(
	ctx context.Context,
	mainWallet *keypair.Full,
	sponsorWallet *keypair.Full,
	lockedBalanceID string,
	transferAddress string,
	amount string,
	unlockTime time.Time,
) error {
	
	// Start both operations simultaneously at unlock time
	var wg sync.WaitGroup
	
	// Claim operation (sponsored)
	wg.Add(1)
	go func() {
		defer wg.Done()
		cp.executeClaimOperation(ctx, mainWallet, sponsorWallet, lockedBalanceID, unlockTime)
	}()
	
	// Transfer operation (independent)
	wg.Add(1)
	go func() {
		defer wg.Done()
		cp.executeTransferOperation(ctx, mainWallet, transferAddress, amount, unlockTime)
	}()
	
	wg.Wait()
	
	fmt.Printf("🚀 Operations completed: Claims=%d, Transfers=%d, Successes=%d\n",
		atomic.LoadInt64(&cp.claimAttempts),
		atomic.LoadInt64(&cp.transferAttempts),
		atomic.LoadInt64(&cp.successCount))
	
	return nil
}

// Execute claiming operation with sponsor fee payment
func (cp *ConcurrentProcessor) executeClaimOperation(
	ctx context.Context,
	mainWallet *keypair.Full,
	sponsorWallet *keypair.Full,
	lockedBalanceID string,
	unlockTime time.Time,
) {
	
	// Wait until exact unlock time
	time.Sleep(time.Until(unlockTime))
	
	// Execute 1000+ concurrent claim attempts
	for i := 0; i < cp.maxConcurrentOps; i++ {
		go func(attemptID int) {
			defer func() {
				atomic.AddInt64(&cp.claimAttempts, 1)
			}()
			
			// Continuous retry until success
			for {
				select {
				case <-ctx.Done():
					return
				default:
					err := cp.wallet.ClaimWithSponsor(mainWallet, sponsorWallet, lockedBalanceID)
					if err == nil {
						atomic.AddInt64(&cp.successCount, 1)
						fmt.Printf("✅ Claim SUCCESS #%d\n", attemptID)
						return
					}
					
					time.Sleep(cp.retryInterval)
				}
			}
		}(i)
	}
}

// Execute transfer operation (independent of claiming)
func (cp *ConcurrentProcessor) executeTransferOperation(
	ctx context.Context,
	mainWallet *keypair.Full,
	transferAddress string,
	amount string,
	unlockTime time.Time,
) {
	
	// Wait until exact unlock time (same as claim)
	time.Sleep(time.Until(unlockTime))
	
	// Execute 1000+ concurrent transfer attempts
	for i := 0; i < cp.maxConcurrentOps; i++ {
		go func(attemptID int) {
			defer func() {
				atomic.AddInt64(&cp.transferAttempts, 1)
			}()
			
			// Continuous retry until success (independent of claim status)
			for {
				select {
				case <-ctx.Done():
					return
				default:
					// Always attempt transfer regardless of available balance
					err := cp.wallet.TransferWithHighFee(mainWallet, amount, transferAddress)
					if err == nil {
						atomic.AddInt64(&cp.successCount, 1)
						fmt.Printf("✅ Transfer SUCCESS #%d\n", attemptID)
						return
					}
					
					time.Sleep(cp.retryInterval)
				}
			}
		}(i)
	}
}

// Monitor operation progress
func (cp *ConcurrentProcessor) GetOperationStats() OperationStats {
	return OperationStats{
		ClaimAttempts:    atomic.LoadInt64(&cp.claimAttempts),
		TransferAttempts: atomic.LoadInt64(&cp.transferAttempts),
		SuccessCount:     atomic.LoadInt64(&cp.successCount),
		ActiveOps:        cp.countActiveOps(),
	}
}

func (cp *ConcurrentProcessor) countActiveOps() int {
	count := 0
	cp.activeOps.Range(func(key, value interface{}) bool {
		count++
		return true
	})
	return count
}

type OperationStats struct {
	ClaimAttempts    int64 `json:"claim_attempts"`
	TransferAttempts int64 `json:"transfer_attempts"`
	SuccessCount     int64 `json:"success_count"`
	ActiveOps        int   `json:"active_ops"`
}