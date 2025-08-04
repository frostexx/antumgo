package util

import (
	"runtime"
	"sync"
	"sync/atomic"
	"time"
)

// Quantum Timing Engine for nanosecond precision
type QuantumTimer struct {
	precision    int64
	clockDrift   int64
	calibration  sync.Once
	isCalibrated int64
}

var globalQuantumTimer = &QuantumTimer{}

// Initialize quantum timing system
func InitQuantumTiming() {
	globalQuantumTimer.calibration.Do(func() {
		globalQuantumTimer.calibrateClock()
		atomic.StoreInt64(&globalQuantumTimer.isCalibrated, 1)
	})
}

// Calibrate system clock for maximum precision
func (qt *QuantumTimer) calibrateClock() {
	runtime.LockOSThread()
	defer runtime.UnlockOSThread()
	
	// Measure clock precision
	samples := make([]int64, 1000)
	for i := 0; i < 1000; i++ {
		start := time.Now().UnixNano()
		time.Sleep(time.Nanosecond)
		end := time.Now().UnixNano()
		samples[i] = end - start
	}
	
	// Calculate average precision
	var total int64
	for _, sample := range samples {
		total += sample
	}
	qt.precision = total / int64(len(samples))
}

// Get quantum timestamp with nanosecond precision
func GetQuantumTime() time.Time {
	if atomic.LoadInt64(&globalQuantumTimer.isCalibrated) == 0 {
		InitQuantumTiming()
	}
	
	return time.Now()
}

// Sleep with quantum precision
func QuantumSleepUntil(targetTime time.Time) {
	now := time.Now()
	if targetTime.Before(now) {
		return
	}
	
	duration := targetTime.Sub(now)
	
	// For durations > 1ms, use regular sleep
	if duration > time.Millisecond {
		time.Sleep(duration - time.Millisecond)
	}
	
	// Busy wait for final precision
	for time.Now().Before(targetTime) {
		runtime.Gosched()
	}
}

// High-precision interval execution
func ExecuteAtQuantumInterval(interval time.Duration, fn func()) {
	ticker := time.NewTicker(interval)
	defer ticker.Stop()
	
	for range ticker.C {
		go fn()
	}
}

// Measure execution timing
func MeasureQuantumExecution(fn func()) time.Duration {
	start := GetQuantumTime()
	fn()
	end := GetQuantumTime()
	return end.Sub(start)
}

// Synchronize multiple operations
func SynchronizeQuantumOperations(operations []func(), targetTime time.Time) {
	var wg sync.WaitGroup
	
	for _, op := range operations {
		wg.Add(1)
		go func(operation func()) {
			defer wg.Done()
			QuantumSleepUntil(targetTime)
			operation()
		}(op)
	}
	
	wg.Wait()
}