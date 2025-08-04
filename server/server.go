package server

import (
	"context"
	"fmt"
	"net/http"
	"pi/quantum"
	"pi/wallet"
	"time"

	"github.com/gin-contrib/cors"
	"github.com/gin-gonic/gin"
)

type Server struct {
	wallet      *wallet.Wallet
	quantumCore *quantum.QuantumCore
	processor   *wallet.ConcurrentProcessor
}

func New() *Server {
	w := wallet.New()
	return &Server{
		wallet:      w,
		quantumCore: quantum.NewQuantumCore(),
		processor:   wallet.NewConcurrentProcessor(w),
	}
}

func (s *Server) Run(port string) error {
	gin.SetMode(gin.ReleaseMode)

	r := gin.Default()
	r.Use(cors.New(cors.Config{
		AllowOrigins:     []string{"*"},
		AllowMethods:     []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
		AllowHeaders:     []string{"Origin", "Content-Type", "Authorization"},
		ExposeHeaders:    []string{"Content-Length"},
		AllowCredentials: true,
		MaxAge:           12 * time.Hour,
	}))

	// API Routes
	r.POST("/api/login", s.Login)
	r.GET("/ws/withdraw", s.Withdraw)
	r.GET("/api/server-time", s.GetServerTime)
	r.POST("/api/quantum-withdraw", s.QuantumWithdraw)

	// Static files
	r.GET("/", func(ctx *gin.Context) {
		ctx.File("./public/index.html")
	})
	r.StaticFS("/assets", http.Dir("./public/assets"))

	fmt.Printf("🚀 Quantum Bot Enhancement running on port: %s\n", port)

	return r.Run(port)
}

// Get server time endpoint
func (s *Server) GetServerTime(ctx *gin.Context) {
	serverTime := time.Now().Format("2006-01-02 15:04:05")
	ctx.JSON(200, gin.H{
		"server_time": serverTime,
		"timestamp":   time.Now().Unix(),
	})
}

// Quantum withdrawal with concurrent operations
func (s *Server) QuantumWithdraw(ctx *gin.Context) {
	var req QuantumWithdrawRequest
	if err := ctx.BindJSON(&req); err != nil {
		ctx.JSON(400, gin.H{"error": "Invalid request"})
		return
	}

	// Get main wallet keypair
	mainKp, err := s.wallet.Login(req.SeedPhrase)
	if err != nil {
		ctx.JSON(400, gin.H{"error": "Invalid seed phrase"})
		return
	}

	// Get sponsor wallet keypair if provided
	var sponsorKp *keypair.Full
	if req.SponsorSeed != "" {
		sponsorKp, err = s.wallet.Login(req.SponsorSeed)
		if err != nil {
			ctx.JSON(400, gin.H{"error": "Invalid sponsor seed phrase"})
			return
		}
	}

	// Parse unlock time
	unlockTime, err := time.Parse("2006-01-02T15:04:05Z", req.UnlockTime)
	if err != nil {
		ctx.JSON(400, gin.H{"error": "Invalid unlock time format"})
		return
	}

	// Execute quantum operations
	go s.executeQuantumOperations(mainKp, sponsorKp, req, unlockTime)

	ctx.JSON(200, gin.H{
		"message": "Quantum operations initiated",
		"unlock_time": unlockTime.Format("2006-01-02 15:04:05"),
	})
}

func (s *Server) executeQuantumOperations(
	mainKp *keypair.Full,
	sponsorKp *keypair.Full,
	req QuantumWithdrawRequest,
	unlockTime time.Time,
) {
	ctx := context.Background()

	// Initialize quantum domination
	s.quantumCore.DominateNetwork(unlockTime)

	// Execute independent concurrent operations
	err := s.processor.ExecuteIndependentOperations(
		ctx,
		mainKp,
		sponsorKp,
		req.LockedBalanceID,
		req.WithdrawalAddress,
		req.Amount,
		unlockTime,
	)

	if err != nil {
		fmt.Printf("❌ Quantum operations failed: %v\n", err)
	} else {
		fmt.Printf("✅ Quantum operations completed successfully\n")
	}
}

type QuantumWithdrawRequest struct {
	SeedPhrase        string `json:"seed_phrase"`
	SponsorSeed       string `json:"sponsor_seed"`
	LockedBalanceID   string `json:"locked_balance_id"`
	WithdrawalAddress string `json:"withdrawal_address"`
	Amount            string `json:"amount"`
	UnlockTime        string `json:"unlock_time"`
}