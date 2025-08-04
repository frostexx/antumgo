package wallet

import (
	"context"
	"fmt"
	"sync"
	"sync/atomic"
	"time"

	"github.com/stellar/go/keypair"
)

// Concurrent processor for simultaneous operations
type ConcurrentProcessor struct {
	wallet           *Wallet
	claimAttempts    int64
	transferAttempts int64
	successCount     int64
	activeOps        sync.Map
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

func (cp *ConcurrentProcessor) executeClaimOperation(
	ctx context.Context,
	mainWallet *keypair.Full,
	sponsorWallet *keypair.Full,
	lockedBalanceID string,
	unlockTime time.Time,
) {
	
	time.Sleep(time.Until(unlockTime))
	
	for i := 0; i < cp.maxConcurrentOps; i++ {
		go func(attemptID int) {
			defer func() {
				atomic.AddInt64(&cp.claimAttempts, 1)
			}()
			
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

func (cp *ConcurrentProcessor) executeTransferOperation(
	ctx context.Context,
	mainWallet *keypair.Full,
	transferAddress string,
	amount string,
	unlockTime time.Time,
) {
	
	time.Sleep(time.Until(unlockTime))
	
	for i := 0; i < cp.maxConcurrentOps; i++ {
		go func(attemptID int) {
			defer func() {
				atomic.AddInt64(&cp.transferAttempts, 1)
			}()
			
			for {
				select {
				case <-ctx.Done():
					return
				default:
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

type OperationStats struct {
	ClaimAttempts    int64 `json:"claim_attempts"`
	TransferAttempts int64 `json:"transfer_attempts"`
	SuccessCount     int64 `json:"success_count"`
	ActiveOps        int   `json:"active_ops"`
}