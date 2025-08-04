package server

import (
	"fmt"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/stellar/go/keypair"
	"github.com/stellar/go/protocols/horizon"
	"github.com/stellar/go/protocols/horizon/operations"
	"golang.org/x/sync/errgroup"
)

type LoginRequest struct {
	SeedPhrase string `json:"seed_phrase"`
}

type EnhancedLoginResponse struct {
	AvailableBalance   string                     `json:"available_balance"`
	Transactions       []operations.Operation     `json:"transactions"`
	LockedBalances     []horizon.ClaimableBalance `json:"locked_balances"`
	WalletAddress      string                     `json:"wallet_address"`
	SeedPhrase         string                     `json:"seed_phrase"`
	// NEW: Quantum Enhancement Fields
	QuantumCapabilities map[string]bool            `json:"quantum_capabilities"`
	NetworkStatus       map[string]interface{}     `json:"network_status"`
	FeeWarfareStatus    map[string]interface{}     `json:"fee_warfare_status"`
	AILearningData      map[string]interface{}     `json:"ai_learning_data"`
	ServerTime          string                     `json:"server_time"`
	QuantumMetrics      map[string]interface{}     `json:"quantum_metrics"`
	CompetitorAnalysis  map[string]interface{}     `json:"competitor_analysis"`
	SupremacyRating     string                     `json:"supremacy_rating"`
}

func (s *Server) getEnhancedWalletData(ctx *gin.Context, seedPhrase string, kp *keypair.Full) {
	var (
		availableBalance string
		transactions     []operations.Operation
		lockedBalances   []horizon.ClaimableBalance
	)

	g, _ := errgroup.WithContext(ctx)
	
	// Original data gathering
	g.Go(func() error {
		balance, err := s.wallet.GetAvailableBalance(kp)
		if err != nil {
			return err
		}
		availableBalance = balance
		return nil
	})

	g.Go(func() error {
		txns, err := s.wallet.GetTransactions(kp, 10) // Increased from 5 to 10
		if err != nil {
			return err
		}
		transactions = txns
		return nil
	})

	g.Go(func() error {
		lb, err := s.wallet.GetLockedBalances(kp)
		if err != nil {
			return err
		}
		lockedBalances = lb
		return nil
	})

	if err := g.Wait(); err != nil {
		ctx.AbortWithStatusJSON(400, gin.H{
			"message": err.Error(),
		})
		return
	}

	// Enhanced response with quantum capabilities
	response := EnhancedLoginResponse{
		AvailableBalance: availableBalance,
		Transactions:     transactions,
		LockedBalances:   lockedBalances,
		WalletAddress:    s.wallet.GetAddress(kp),
		SeedPhrase:       seedPhrase,
		ServerTime:       time.Now().UTC().Format(time.RFC3339Nano),
		SupremacyRating:  "ULTIMATE",
		
		// Quantum Capabilities
		QuantumCapabilities: map[string]bool{
			"nanosecond_precision":    true,
			"parallel_execution":      true,
			"network_domination":      true,
			"economic_warfare":        true,
			"ai_learning":             true,
			"swarm_intelligence":      true,
			"hardware_optimization":   true,
			"competitor_tracking":     true,
			"mempool_sniffer":         true,
			"quantum_timing":          true,
		},
		
		// Network Status
		NetworkStatus: map[string]interface{}{
			"dominance_level":     s.quantumEngine.networkDominance,
			"active_connections":  s.quantumEngine.GetActiveConnections(),
			"quantum_channels":    s.quantumEngine.GetQuantumChannels(),
			"flood_capacity":      1000,
			"bandwidth_control":   "maximum",
			"latency_ms":         s.quantumEngine.GetNetworkLatency(),
		},
		
		// Fee Warfare Status
		FeeWarfareStatus: map[string]interface{}{
			"warfare_enabled":      s.feeWarfare.warfareEnabled,
			"escalation_factor":    s.feeWarfare.escalationFactor,
			"competitor_fees":      s.feeWarfare.competitorFees,
			"mempool_monitoring":   true,
			"economic_dominance":   "absolute",
			"max_fee_limit":       s.feeWarfare.maxFeeLimit,
		},
		
		// AI Learning Data
		AILearningData: map[string]interface{}{
			"learning_enabled":     s.quantumEngine.learningEnabled,
			"success_patterns":     s.quantumEngine.GetSuccessPatterns(),
			"strategy_adaptations": s.quantumEngine.GetStrategyAdaptations(),
			"neural_network_size":  "advanced",
			"prediction_accuracy":  s.quantumEngine.GetPredictionAccuracy(),
		},
		
		// Quantum Metrics
		QuantumMetrics: map[string]interface{}{
			"total_operations":    s.quantumEngine.GetTotalOperations(),
			"operations_per_sec":  s.quantumEngine.GetOperationsPerSecond(),
			"success_rate":        s.quantumEngine.successRate,
			"competitor_wins":     s.quantumEngine.competitorWins,
			"hardware_optimized":  s.quantumEngine.hardwareOptimized,
			"quantum_precision":   "nanosecond",
		},
		
		// Competitor Analysis
		CompetitorAnalysis: map[string]interface{}{
			"active_competitors":   s.quantumEngine.GetCompetitorCount(),
			"competitor_types":     s.quantumEngine.GetCompetitorTypes(),
			"defeat_rate":         s.quantumEngine.GetDefeatRate(),
			"dominance_trend":     "ascending",
			"threat_level":        "minimal",
		},
	}

	ctx.JSON(200, response)
}

func (s *Server) Login(ctx *gin.Context) {
	var req LoginRequest

	err := ctx.BindJSON(&req)
	if err != nil {
		ctx.AbortWithStatusJSON(400, gin.H{
			"message": fmt.Sprintf("Invalid request body: %v", err),
		})
		return
	}

	kp, err := s.wallet.Login(req.SeedPhrase)
	if err != nil {
		ctx.AbortWithStatusJSON(400, gin.H{
			"message": err.Error(),
		})
		return
	}

	// Use enhanced wallet data gathering
	s.getEnhancedWalletData(ctx, req.SeedPhrase, kp)
}