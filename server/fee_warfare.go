package server

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"sync"
	"time"
)

type FeeWarfareEngine struct {
	// Mempool monitoring
	mempoolSniffer   *MempoolSniffer
	competitorFees   map[string]int64
	feeHistory       []FeeSnapshot
	
	// Economic warfare
	escalationFactor float64
	maxFeeLimit      int64
	warfareEnabled   bool
	
	// Real-time intelligence
	competitorTxns   []CompetitorTransaction
	networkCongestion float64
	
	mu sync.RWMutex
}

type FeeSnapshot struct {
	Timestamp    time.Time
	Operation    string
	CompetitorFee int64
	OurFee       int64
	Success      bool
}

type CompetitorTransaction struct {
	Hash      string
	Fee       int64
	Timestamp time.Time
	Status    string
	BotID     string
}

type MempoolSniffer struct {
	client   *http.Client
	endpoint string
	active   bool
}

func NewFeeWarfareEngine() *FeeWarfareEngine {
	return &FeeWarfareEngine{
		mempoolSniffer:   NewMempoolSniffer(),
		competitorFees:   make(map[string]int64),
		feeHistory:       make([]FeeSnapshot, 0),
		escalationFactor: 2.5, // 250% escalation
		maxFeeLimit:      50000000, // 50M PI maximum
		warfareEnabled:   true,
		competitorTxns:   make([]CompetitorTransaction, 0),
	}
}

func NewMempoolSniffer() *MempoolSniffer {
	return &MempoolSniffer{
		client: &http.Client{
			Timeout: 100 * time.Millisecond,
		},
		endpoint: "https://api.mainnet.minepi.com/mempool",
		active:   true,
	}
}

// MEMPOOL SNIFFER: Real-time competitor monitoring
func (fwe *FeeWarfareEngine) StartMempoolMonitoring(ctx context.Context) {
	go func() {
		ticker := time.NewTicker(50 * time.Millisecond) // Ultra-fast monitoring
		defer ticker.Stop()
		
		for {
			select {
			case <-ctx.Done():
				return
			case <-ticker.C:
				fwe.sniffMempool()
			}
		}
	}()
}

func (fwe *FeeWarfareEngine) sniffMempool() {
	resp, err := fwe.mempoolSniffer.client.Get(fwe.mempoolSniffer.endpoint)
	if err != nil {
		return
	}
	defer resp.Body.Close()
	
	var mempoolData struct {
		Transactions []struct {
			Hash      string `json:"hash"`
			Fee       int64  `json:"fee"`
			Timestamp string `json:"timestamp"`
			Type      string `json:"type"`
		} `json:"transactions"`
	}
	
	if err := json.NewDecoder(resp.Body).Decode(&mempoolData); err != nil {
		return
	}
	
	fwe.mu.Lock()
	defer fwe.mu.Unlock()
	
	// Analyze competitor transactions
	for _, txn := range mempoolData.Transactions {
		if fwe.isCompetitorTransaction(txn.Hash, txn.Fee) {
			timestamp, _ := time.Parse(time.RFC3339, txn.Timestamp)
			
			competitorTxn := CompetitorTransaction{
				Hash:      txn.Hash,
				Fee:       txn.Fee,
				Timestamp: timestamp,
				Status:    "pending",
				BotID:     fwe.identifyCompetitorBot(txn.Fee),
			}
			
			fwe.competitorTxns = append(fwe.competitorTxns, competitorTxn)
			fwe.updateCompetitorFees(txn.Type, txn.Fee)
		}
	}
}

// ECONOMIC WARFARE: Calculate dominant fees
func (fwe *FeeWarfareEngine) CalculateWarfareFee(operation string, urgency float64) int64 {
	fwe.mu.RLock()
	defer fwe.mu.RUnlock()
	
	if !fwe.warfareEnabled {
		return fwe.getBaseFee(operation)
	}
	
	// Get highest competitor fee for this operation
	competitorFee := fwe.competitorFees[operation]
	if competitorFee == 0 {
		competitorFee = fwe.getBaseFee(operation)
	}
	
	// Economic warfare escalation
	warfareFee := int64(float64(competitorFee) * fwe.escalationFactor)
	
	// Urgency multiplier (1.0 = normal, 2.0 = critical)
	urgencyFee := int64(float64(warfareFee) * urgency)
	
	// Network congestion adjustment
	congestionMultiplier := 1.0 + fwe.networkCongestion
	finalFee := int64(float64(urgencyFee) * congestionMultiplier)
	
	// Enforce maximum limit
	if finalFee > fwe.maxFeeLimit {
		finalFee = fwe.maxFeeLimit
	}
	
	// Record our fee strategy
	fwe.recordFeeSnapshot(operation, competitorFee, finalFee)
	
	log.Printf("WARFARE FEE CALCULATED: %s = %d PI (competitor: %d, escalation: %.1fx)", 
		operation, finalFee, competitorFee, fwe.escalationFactor)
	
	return finalFee
}

// COMPETITOR INTELLIGENCE: Identify and track bots
func (fwe *FeeWarfareEngine) isCompetitorTransaction(hash string, fee int64) bool {
	// Identify competitor patterns
	// High fees (>1M PI) indicate bot activity
	if fee > 1000000 {
		return true
	}
	
	// Pattern recognition for known competitor bots
	competitorPatterns := []int64{
		3200000, // Known competitor fee
		9400000, // Known competitor fee
		5000000, // Common bot fee
		7500000, // Common bot fee
	}
	
	for _, pattern := range competitorPatterns {
		if fee == pattern {
			return true
		}
	}
	
	return false
}

func (fwe *FeeWarfareEngine) identifyCompetitorBot(fee int64) string {
	// Bot identification based on fee patterns
	switch {
	case fee == 3200000:
		return "RustBot-Alpha"
	case fee == 9400000:
		return "GoBot-Omega"
	case fee >= 5000000 && fee < 8000000:
		return "PythonBot-Beta"
	case fee >= 8000000:
		return "NodeBot-Gamma"
	default:
		return "UnknownBot"
	}
}

func (fwe *FeeWarfareEngine) updateCompetitorFees(operation string, fee int64) {
	if currentFee, exists := fwe.competitorFees[operation]; !exists || fee > currentFee {
		fwe.competitorFees[operation] = fee
	}
}

func (fwe *FeeWarfareEngine) getBaseFee(operation string) int64 {
	baseFees := map[string]int64{
		"claim":    9400000, // Start with known competitor maximum
		"transfer": 3200000, // Start with known competitor maximum
		"payment":  1000000, // Standard payment fee
	}
	
	if fee, exists := baseFees[operation]; exists {
		return fee
	}
	return 1000000 // Default fee
}

func (fwe *FeeWarfareEngine) recordFeeSnapshot(operation string, competitorFee, ourFee int64) {
	snapshot := FeeSnapshot{
		Timestamp:     time.Now(),
		Operation:     operation,
		CompetitorFee: competitorFee,
		OurFee:        ourFee,
		Success:       false, // Will be updated later
	}
	
	fwe.feeHistory = append(fwe.feeHistory, snapshot)
	
	// Keep only last 1000 snapshots
	if len(fwe.feeHistory) > 1000 {
		fwe.feeHistory = fwe.feeHistory[1:]
	}
}

// WARFARE ANALYTICS: Get real-time intelligence
func (fwe *FeeWarfareEngine) GetWarfareAnalytics() map[string]interface{} {
	fwe.mu.RLock()
	defer fwe.mu.RUnlock()
	
	return map[string]interface{}{
		"competitor_fees":     fwe.competitorFees,
		"escalation_factor":   fwe.escalationFactor,
		"network_congestion":  fwe.networkCongestion,
		"active_competitors":  len(fwe.competitorTxns),
		"warfare_enabled":     fwe.warfareEnabled,
		"recent_fee_history":  fwe.getRecentFeeHistory(10),
	}
}

func (fwe *FeeWarfareEngine) getRecentFeeHistory(limit int) []FeeSnapshot {
	if len(fwe.feeHistory) <= limit {
		return fwe.feeHistory
	}
	return fwe.feeHistory[len(fwe.feeHistory)-limit:]
}

// NETWORK CONGESTION: Monitor and adapt
func (fwe *FeeWarfareEngine) UpdateNetworkCongestion(congestion float64) {
	fwe.mu.Lock()
	defer fwe.mu.Unlock()
	
	fwe.networkCongestion = congestion
}

// WARFARE CONTROL: Enable/disable economic warfare
func (fwe *FeeWarfareEngine) SetWarfareMode(enabled bool) {
	fwe.mu.Lock()
	defer fwe.mu.Unlock()
	
	fwe.warfareEnabled = enabled
	log.Printf("Economic warfare mode: %v", enabled)
}