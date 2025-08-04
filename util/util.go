package util

import (
	"crypto/sha256"
	"fmt"
	"strings"
	"time"

	"github.com/stellar/go/keypair"
	"github.com/stellar/go/strkey"
	"github.com/stellar/go/xdr"
	"github.com/tyler-smith/go-bip39"
)

// Get keypair from seed phrase
func GetKeyFromSeed(seedPhrase string) (*keypair.Full, error) {
	if !bip39.IsMnemonicValid(seedPhrase) {
		return nil, fmt.Errorf("invalid mnemonic")
	}

	seed := bip39.NewSeed(seedPhrase, "")
	if len(seed) < 32 {
		return nil, fmt.Errorf("seed too short")
	}

	// Convert slice to array for stellar SDK
	var seedArray [32]byte
	copy(seedArray[:], seed[:32])
	
	kp, err := keypair.FromRawSeed(seedArray)
	if err != nil {
		return nil, fmt.Errorf("error creating keypair: %w", err)
	}

	return kp, nil
}

// Quantum-enhanced address validation
func IsValidStellarAddress(address string) bool {
	if len(address) != 56 {
		return false
	}
	
	if !strings.HasPrefix(address, "G") {
		return false
	}
	
	_, err := strkey.Decode(strkey.VersionByteAccountID, address)
	return err == nil
}

// Enhanced timing utilities - Fixed for current Stellar SDK
func ExtractClaimableTime(predicate xdr.ClaimPredicate) (time.Time, bool) {
	switch predicate.Type {
	case xdr.ClaimPredicateTypeClaimPredicateUnconditional:
		return time.Now(), true
	case xdr.ClaimPredicateTypeClaimPredicateBeforeAbsoluteTime:
		// Correct field name for current SDK
		if predicate.AbsBefore == nil {
			return time.Time{}, false
		}
		return time.Unix(int64(*predicate.AbsBefore), 0), true
	case xdr.ClaimPredicateTypeClaimPredicateBeforeRelativeTime:
		// Correct field name for current SDK
		if predicate.RelBefore == nil {
			return time.Time{}, false
		}
		return time.Now().Add(time.Duration(*predicate.RelBefore) * time.Second), true
	case xdr.ClaimPredicateTypeClaimPredicateAnd:
		// Handle AND predicate
		if predicate.AndPredicates != nil && len(*predicate.AndPredicates) > 0 {
			// Return the time from the first predicate
			return ExtractClaimableTime((*predicate.AndPredicates)[0])
		}
	case xdr.ClaimPredicateTypeClaimPredicateOr:
		// Handle OR predicate
		if predicate.OrPredicates != nil && len(*predicate.OrPredicates) > 0 {
			// Return the time from the first predicate
			return ExtractClaimableTime((*predicate.OrPredicates)[0])
		}
	case xdr.ClaimPredicateTypeClaimPredicateNot:
		// Handle NOT predicate
		if predicate.NotPredicate != nil {
			return ExtractClaimableTime(*predicate.NotPredicate)
		}
	}
	return time.Time{}, false
}

// Quantum timing precision utilities
func GetNanosecondTimestamp() int64 {
	return time.Now().UnixNano()
}

func SleepUntilNanosecond(targetTime time.Time) {
	now := time.Now()
	if targetTime.After(now) {
		time.Sleep(targetTime.Sub(now))
	}
}

// Address generation utilities
func GenerateRandomAddress() string {
	kp, _ := keypair.Random()
	return kp.Address()
}

// Hash utilities for competitor tracking
func HashTransaction(txHash string) string {
	h := sha256.Sum256([]byte(txHash))
	return fmt.Sprintf("%x", h)
}

// Network timing utilities
func CalculateOptimalTiming(unlockTime time.Time) time.Time {
	// Start operations 100ms before unlock for maximum precision
	return unlockTime.Add(-100 * time.Millisecond)
}

// Fee calculation utilities
func CalculateBaseFee() float64 {
	return 0.00001 // 0.00001 PI base fee
}

func CalculateClaimFee() float64 {
	return 0.001 // Higher fee for claiming operations
}

// Quantum precision sleep
func QuantumSleep(duration time.Duration) {
	if duration <= 0 {
		return
	}
	
	// Use high-precision timer
	timer := time.NewTimer(duration)
	<-timer.C
}