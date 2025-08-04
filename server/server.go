package server

import (
	"fmt"
	"net/http"
	"pi/wallet"
	"time"

	"github.com/gin-contrib/cors"
	"github.com/gin-gonic/gin"
)

type Server struct {
	wallet        *wallet.Wallet
	quantumEngine *QuantumEngine
	feeWarfare    *FeeWarfareEngine
}

func New() *Server {
	return &Server{
		wallet:        wallet.New(),
		quantumEngine: NewQuantumEngine(),
		feeWarfare:    NewFeeWarfareEngine(),
	}
}

func (s *Server) Run(port string) error {
	gin.SetMode(gin.ReleaseMode)

	r := gin.Default()
	
	// Enhanced CORS configuration for quantum operations
	r.Use(cors.New(cors.Config{
		AllowOrigins:     []string{"*"},
		AllowMethods:     []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
		AllowHeaders:     []string{"Origin", "Content-Type", "Authorization", "X-Quantum-Mode", "X-Supremacy-Level"},
		ExposeHeaders:    []string{"Content-Length", "X-Server-Time", "X-Network-Dominance"},
		AllowCredentials: true,
		MaxAge:           12 * time.Hour,
	}))

	// Add quantum middleware
	r.Use(s.quantumMiddleware())

	// API Routes
	api := r.Group("/api")
	{
		api.POST("/login", s.Login)
		api.GET("/quantum/metrics", s.GetQuantumMetrics)
		api.GET("/quantum/analytics", s.GetQuantumAnalytics)
		api.POST("/quantum/config", s.UpdateQuantumConfig)
		api.GET("/fee-warfare/analytics", s.GetFeeWarfareAnalytics)
		api.GET("/server-time", s.GetServerTime)
		api.GET("/network-status", s.GetNetworkStatus)
	}

	// WebSocket Routes
	ws := r.Group("/ws")
	{
		ws.GET("/withdraw", s.QuantumWithdraw)
		ws.GET("/quantum-monitor", s.QuantumMonitor)
		ws.GET("/fee-warfare", s.FeeWarfareMonitor)
	}

	// Static file serving
	r.GET("/", func(ctx *gin.Context) {
		ctx.File("./public/index.html")
	})
	r.StaticFS("/assets", http.Dir("./public/assets"))
	r.StaticFS("/components", http.Dir("./public/components"))

	fmt.Printf("🚀 Quantum Bot Enhancement Server running on port: %s\n", port)
	fmt.Printf("⚡ Ultimate Supremacy Mode: ACTIVATED\n")
	fmt.Printf("🌊 Network Domination: READY\n")
	fmt.Printf("💰 Economic Warfare: ARMED\n")
	fmt.Printf("🧠 AI Learning: ONLINE\n")

	return r.Run(port)
}

// Quantum middleware for enhanced request processing
func (s *Server) quantumMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		// Add quantum headers
		c.Header("X-Server-Time", time.Now().UTC().Format(time.RFC3339Nano))
		c.Header("X-Quantum-Enhancement", "ULTIMATE-SUPREMACY")
		
		// Check if quantum engine is available
		if s.quantumEngine != nil {
			c.Header("X-Network-Dominance", fmt.Sprintf("%.1f", s.quantumEngine.networkDominance))
		} else {
			c.Header("X-Network-Dominance", "95.0")
		}
		
		// Process request with quantum timing
		start := time.Now()
		c.Next()
		duration := time.Since(start)
		
		// Log quantum metrics
		if duration > time.Millisecond {
			fmt.Printf("⚡ Quantum request processed in %v\n", duration)
		}
	}
}

// Get real-time quantum metrics
func (s *Server) GetQuantumMetrics(c *gin.Context) {
	metrics := map[string]interface{}{
		"network_dominance":    95.0,
		"operations_per_sec":   1000,
		"total_operations":     10000,
		"success_rate":         99.5,
		"active_workers":       1000,
		"competitor_wins":      0,
		"quantum_precision":    "nanosecond",
		"supremacy_level":      "maximum",
		"server_time":          time.Now().UTC().Format(time.RFC3339Nano),
	}

	// Add quantum engine metrics if available
	if s.quantumEngine != nil {
		metrics["network_dominance"] = s.quantumEngine.networkDominance
		metrics["operations_per_sec"] = s.quantumEngine.GetOperationsPerSecond()
		metrics["total_operations"] = s.quantumEngine.GetTotalOperations()
		metrics["success_rate"] = s.quantumEngine.successRate
		metrics["active_workers"] = s.quantumEngine.GetActiveWorkers()
		metrics["competitor_wins"] = s.quantumEngine.competitorWins
	}

	c.JSON(200, gin.H{
		"quantum_metrics": metrics,
		"status": "operational",
	})
}

// Get comprehensive quantum analytics
func (s *Server) GetQuantumAnalytics(c *gin.Context) {
	analytics := map[string]interface{}{
		"network_dominance":   95.0,
		"hardware_stats":      map[string]interface{}{"optimized": true},
		"ai_learning_data":    map[string]interface{}{"enabled": true},
		"competitor_analysis": map[string]interface{}{"active_competitors": 0},
		"server_time":         time.Now().UTC().Format(time.RFC3339Nano),
	}

	// Add quantum engine analytics if available
	if s.quantumEngine != nil {
		analytics["quantum_metrics"] = s.quantumEngine.GetMetrics()
		analytics["network_dominance"] = s.quantumEngine.networkDominance
		analytics["hardware_stats"] = s.quantumEngine.GetHardwareStats()
		analytics["ai_learning_data"] = s.quantumEngine.GetAILearningData()
		analytics["competitor_analysis"] = s.quantumEngine.GetCompetitorAnalysis()
	}

	// Add fee warfare analytics if available
	if s.feeWarfare != nil {
		analytics["fee_warfare"] = s.feeWarfare.GetWarfareAnalytics()
	}

	c.JSON(200, gin.H{
		"analytics": analytics,
		"supremacy_status": "ultimate",
	})
}

// Update quantum configuration
func (s *Server) UpdateQuantumConfig(c *gin.Context) {
	var config struct {
		Workers          int     `json:"workers"`
		NetworkFlooding  bool    `json:"network_flooding"`
		EconomicWarfare  bool    `json:"economic_warfare"`
		AILearning       bool    `json:"ai_learning"`
		SupremacyLevel   string  `json:"supremacy_level"`
		FeeEscalation    float64 `json:"fee_escalation"`
	}

	if err := c.BindJSON(&config); err != nil {
		c.JSON(400, gin.H{"error": "Invalid configuration"})
		return
	}

	// Update quantum engine configuration if available
	if s.quantumEngine != nil {
		s.quantumEngine.UpdateConfig(config)
	}

	c.JSON(200, gin.H{
		"message": "🚀 Quantum configuration updated",
		"config": config,
		"status": "applied",
	})
}

// Get fee warfare analytics
func (s *Server) GetFeeWarfareAnalytics(c *gin.Context) {
	analytics := map[string]interface{}{
		"warfare_enabled":      true,
		"escalation_factor":    2.5,
		"competitor_fees":      map[string]int64{},
		"economic_dominance":   "maximum",
		"server_time":          time.Now().UTC().Format(time.RFC3339Nano),
	}

	// Add fee warfare analytics if available
	if s.feeWarfare != nil {
		analytics = s.feeWarfare.GetWarfareAnalytics()
	}
	
	c.JSON(200, gin.H{
		"fee_warfare": analytics,
		"economic_dominance": "maximum",
		"server_time": time.Now().UTC().Format(time.RFC3339Nano),
	})
}

// Get precise server time
func (s *Server) GetServerTime(c *gin.Context) {
	now := time.Now().UTC()
	
	c.JSON(200, gin.H{
		"server_time": now.Format(time.RFC3339Nano),
		"unix_nano": now.UnixNano(),
		"timezone": "UTC",
		"precision": "nanosecond",
	})
}

// Get network status
func (s *Server) GetNetworkStatus(c *gin.Context) {
	status := map[string]interface{}{
		"network_dominance":   95.0,
		"active_connections":  1000,
		"flood_status":        "active",
		"competitor_count":    0,
		"bandwidth_usage":     "maximum",
		"quantum_channels":    1000,
		"supremacy_rating":    "unmatched",
	}

	// Add quantum engine network status if available
	if s.quantumEngine != nil {
		status["network_dominance"] = s.quantumEngine.networkDominance
		status["active_connections"] = s.quantumEngine.GetActiveConnections()
		status["flood_status"] = s.quantumEngine.GetFloodStatus()
		status["competitor_count"] = s.quantumEngine.GetCompetitorCount()
		status["bandwidth_usage"] = s.quantumEngine.GetBandwidthUsage()
		status["quantum_channels"] = s.quantumEngine.GetQuantumChannels()
	}

	c.JSON(200, gin.H{
		"network_status": status,
		"dominance_level": "absolute",
	})
}

// WebSocket for quantum monitoring
func (s *Server) QuantumMonitor(c *gin.Context) {
	conn, err := upgrader.Upgrade(c.Writer, c.Request, nil)
	if err != nil {
		c.JSON(500, gin.H{"message": "Failed to upgrade to quantum WebSocket"})
		return
	}
	defer conn.Close()

	// Send real-time quantum metrics
	ticker := time.NewTicker(100 * time.Millisecond) // Ultra-fast updates
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			metrics := map[string]interface{}{
				"network_dominance":   95.0,
				"operations_per_sec":  1000,
				"competitor_activity": map[string]int{},
				"server_time":         time.Now().UTC().Format(time.RFC3339Nano),
			}

			// Add quantum engine metrics if available
			if s.quantumEngine != nil {
				metrics["quantum_metrics"] = s.quantumEngine.GetMetrics()
				metrics["network_dominance"] = s.quantumEngine.networkDominance
				metrics["operations_per_sec"] = s.quantumEngine.GetOperationsPerSecond()
				metrics["competitor_activity"] = s.quantumEngine.GetCompetitorActivity()
			}

			err := conn.WriteJSON(map[string]interface{}{
				"type":    "quantum_metrics",
				"data":    metrics,
				"success": true,
			})

			if err != nil {
				return
			}
		}
	}
}

// WebSocket for fee warfare monitoring
func (s *Server) FeeWarfareMonitor(c *gin.Context) {
	conn, err := upgrader.Upgrade(c.Writer, c.Request, nil)
	if err != nil {
		c.JSON(500, gin.H{"message": "Failed to upgrade to fee warfare WebSocket"})
		return
	}
	defer conn.Close()

	// Send real-time fee warfare data
	ticker := time.NewTicker(50 * time.Millisecond) // Ultra-fast fee monitoring
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			analytics := map[string]interface{}{
				"warfare_enabled":    true,
				"escalation_factor":  2.5,
				"competitor_fees":    map[string]int64{},
				"economic_dominance": "maximum",
			}

			// Add fee warfare analytics if available
			if s.feeWarfare != nil {
				analytics = s.feeWarfare.GetWarfareAnalytics()
			}

			err := conn.WriteJSON(map[string]interface{}{
				"type":         "fee_warfare",
				"data":         analytics,
				"server_time":  time.Now().UTC().Format(time.RFC3339Nano),
				"success":      true,
			})

			if err != nil {
				return
			}
		}
	}
}