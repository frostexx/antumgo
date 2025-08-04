package wallet

import (
	"fmt"
	"os"
	"pi/util"
	"strconv"

	"github.com/stellar/go/clients/horizonclient"
	hClient "github.com/stellar/go/clients/horizonclient"
	"github.com/stellar/go/keypair"
	"github.com/stellar/go/protocols/horizon"
	"github.com/stellar/go/protocols/horizon/operations"
	"github.com/stellar/go/txnbuild"
)

type Wallet struct {
	networkPassphrase string
	serverURL         string
	client            *hClient.Client
	baseReserve       float64
}

func New() *Wallet {
	client := hClient.DefaultPublicNetClient
	client.HorizonURL = os.Getenv("NET_URL")

	w := &Wallet{
		networkPassphrase: os.Getenv("NET_PASSPHRASE"),
		serverURL:         os.Getenv("NET_URL"),
		client:            client,
		baseReserve:       0.49,
	}
	w.GetBaseReserve()

	return w
}

func (w *Wallet) GetBaseReserve() {
	ledger, err := w.client.Ledgers(horizonclient.LedgerRequest{Order: horizonclient.OrderDesc, Limit: 1})
	if err != nil {
		fmt.Println("Error getting base reserve:", err)
		return
	}

	if len(ledger.Embedded.Records) == 0 {
		fmt.Println("No ledger records found")
		return
	}

	baseReserveStr := ledger.Embedded.Records[0].BaseReserve
	w.baseReserve = float64(baseReserveStr) / 1e7
	fmt.Printf("Base reserve: %.7f PI\n", w.baseReserve)
}

func (w *Wallet) Login(seedPhrase string) (*keypair.Full, error) {
	kp, err := util.GetKeyFromSeed(seedPhrase)
	if err != nil {
		return nil, err
	}
	return kp, nil
}

func (w *Wallet) GetAccount(kp *keypair.Full) (horizon.Account, error) {
	accReq := hClient.AccountRequest{AccountID: kp.Address()}
	account, err := w.client.AccountDetail(accReq)
	if err != nil {
		return horizon.Account{}, fmt.Errorf("error fetching account details: %v", err)
	}
	return account, nil
}

func (w *Wallet) GetAvailableBalance(kp *keypair.Full) (string, error) {
	account, err := w.GetAccount(kp)
	if err != nil {
		return "", err
	}

	var totalBalance float64
	for _, b := range account.Balances {
		if b.Type == "native" {
			totalBalance, err = strconv.ParseFloat(b.Balance, 64)
			if err != nil {
				return "", err
			}
			break
		}
	}

	reserve := w.baseReserve * float64(2+account.SubentryCount)
	available := totalBalance - reserve
	if available < 0 {
		available = 0
	}

	return fmt.Sprintf("%.7f", available), nil
}

func (w *Wallet) GetTransactions(kp *keypair.Full, limit uint) ([]operations.Operation, error) {
	opReq := hClient.OperationRequest{
		ForAccount: kp.Address(),
		Limit:      limit,
		Order:      hClient.OrderDesc,
	}
	ops, err := w.client.Operations(opReq)
	if err != nil {
		return nil, fmt.Errorf("error fetching account operations: %v", err)
	}

	return ops.Embedded.Records, nil
}

func (w *Wallet) GetLockedBalances(kp *keypair.Full) ([]horizon.ClaimableBalance, error) {
	cbReq := hClient.ClaimableBalanceRequest{
		Claimant: kp.Address(),
	}
	cbs, err := w.client.ClaimableBalances(cbReq)
	if err != nil {
		return nil, fmt.Errorf("error fetching claimable balances: %v", err)
	}

	return cbs.Embedded.Records, nil
}

func (w *Wallet) GetClaimableBalance(balanceID string) (horizon.ClaimableBalance, error) {
	cbReq := hClient.ClaimableBalanceRequest{ID: balanceID}
	cb, err := w.client.ClaimableBalance(cbReq)
	if err != nil {
		return horizon.ClaimableBalance{}, fmt.Errorf("error fetching claimable balance: %v", err)
	}
	return cb, nil
}

// Claim with sponsor paying fees
func (w *Wallet) ClaimWithSponsor(mainWallet, sponsorWallet *keypair.Full, balanceID string) error {
	// Get sponsor account for sequence number
	sponsorAccount, err := w.GetAccount(sponsorWallet)
	if err != nil {
		return fmt.Errorf("error getting sponsor account: %w", err)
	}

	// Build claim operation
	claimOp := &txnbuild.ClaimClaimableBalance{
		BalanceID: balanceID,
	}

	// Calculate high fee for priority
	baseFee := util.CalculateClaimFee()
	highFee := int64(baseFee * 1000000) // Convert to stroops

	// Build transaction with sponsor paying fees
	tx, err := txnbuild.NewTransaction(
		txnbuild.TransactionParams{
			SourceAccount:        &sponsorAccount,
			IncrementSequenceNum: true,
			Operations:           []txnbuild.Operation{claimOp},
			BaseFee:              highFee,
			Preconditions:        txnbuild.Preconditions{TimeBounds: txnbuild.NewInfiniteTimeout()},
		},
	)
	if err != nil {
		return fmt.Errorf("error building transaction: %w", err)
	}

	// Sign with both wallets
	tx, err = tx.Sign(w.networkPassphrase, sponsorWallet, mainWallet)
	if err != nil {
		return fmt.Errorf("error signing transaction: %w", err)
	}

	// Submit transaction
	_, err = w.client.SubmitTransaction(tx)
	if err != nil {
		return fmt.Errorf("error submitting claim transaction: %w", err)
	}

	fmt.Printf("✅ Claim successful - Balance ID: %s\n", balanceID)
	return nil
}

// Transfer with high fees for priority
func (w *Wallet) TransferWithHighFee(kp *keypair.Full, amountStr string, address string) error {
	// Get account details
	account, err := w.GetAccount(kp)
	if err != nil {
		return fmt.Errorf("error getting account: %w", err)
	}

	// Parse amount
	amount, err := strconv.ParseFloat(amountStr, 64)
	if err != nil {
		return fmt.Errorf("invalid amount: %w", err)
	}

	// Build payment operation
	paymentOp := &txnbuild.Payment{
		Destination: address,
		Amount:      fmt.Sprintf("%.7f", amount),
		Asset:       txnbuild.NativeAsset{},
	}

	// Calculate warfare fee (250% higher)
	baseFee := util.CalculateBaseFee()
	warfareFee := int64(baseFee * 2.5 * 1000000) // Convert to stroops

	// Build transaction
	tx, err := txnbuild.NewTransaction(
		txnbuild.TransactionParams{
			SourceAccount:        &account,
			IncrementSequenceNum: true,
			Operations:           []txnbuild.Operation{paymentOp},
			BaseFee:              warfareFee,
			Preconditions:        txnbuild.Preconditions{TimeBounds: txnbuild.NewInfiniteTimeout()},
		},
	)
	if err != nil {
		return fmt.Errorf("error building transaction: %w", err)
	}

	// Sign transaction
	tx, err = tx.Sign(w.networkPassphrase, kp)
	if err != nil {
		return fmt.Errorf("error signing transaction: %w", err)
	}

	// Submit transaction
	_, err = w.client.SubmitTransaction(tx)
	if err != nil {
		return fmt.Errorf("error submitting transfer transaction: %w", err)
	}

	fmt.Printf("✅ Transfer successful - Amount: %s PI to %s\n", amountStr, address)
	return nil
}