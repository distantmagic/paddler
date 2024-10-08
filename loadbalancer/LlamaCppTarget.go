package loadbalancer

import (
	"net/http/httputil"
	"time"

	"github.com/distantmagic/paddler/llamacpp"
	"github.com/puzpuzpuz/xsync/v3"
)

type LlamaCppTarget struct {
	LastUpdate                    time.Time 						      `json:"last_update"`
	LlamaCppClient                *llamacpp.LlamaCppClient 				  `json:"-"`
	LlamaCppSlotsAggregatedStatus *llamacpp.LlamaCppSlotsAggregatedStatus `json:"llamacpp_slots_aggregated_status"`
	LlamaCppTargetConfiguration   *LlamaCppTargetConfiguration 			  `json:"llamacpp_target_configuration"`
	RBMutex                       xsync.RBMutex 						  `json:"-"`
	RemainingTicksUntilRemoved    int 									  `json:"remaining_ticks_until_removed"`
	ReverseProxy                  *httputil.ReverseProxy 				  `json:"-"`
	TotalUpdates                  int 									  `json:"total_updates"`
}

func (self *LlamaCppTarget) DecrementIdleSlots() {
	self.RBMutex.Lock()
	defer self.RBMutex.Unlock()

	self.LlamaCppSlotsAggregatedStatus.SlotsIdle -= 1
	self.LlamaCppSlotsAggregatedStatus.SlotsProcessing += 1
}

func (self *LlamaCppTarget) DecrementRemainingTicks() {
	self.RBMutex.Lock()
	defer self.RBMutex.Unlock()

	self.RemainingTicksUntilRemoved -= 1
}

func (self *LlamaCppTarget) GetSlotsStatus() (int, int) {
	mutexToken := self.RBMutex.RLock()
	defer self.RBMutex.RUnlock(mutexToken)

	return self.LlamaCppSlotsAggregatedStatus.SlotsIdle, self.LlamaCppSlotsAggregatedStatus.SlotsProcessing
}

func (self *LlamaCppTarget) HasLessSlotsThan(other *LlamaCppTarget) bool {
	mutexToken := self.RBMutex.RLock()
	defer self.RBMutex.RUnlock(mutexToken)

	otherMutexToken := other.RBMutex.RLock()
	defer other.RBMutex.RUnlock(otherMutexToken)

	return self.LlamaCppSlotsAggregatedStatus.SlotsIdle < other.LlamaCppSlotsAggregatedStatus.SlotsIdle
}

func (self *LlamaCppTarget) HasRemainingTicks() bool {
	mutexToken := self.RBMutex.RLock()
	defer self.RBMutex.RUnlock(mutexToken)

	return self.RemainingTicksUntilRemoved > 0
}

func (self *LlamaCppTarget) SetTickStatus(
	lastUpdate time.Time,
	llamaCppSlotsAggregatedStatus *llamacpp.LlamaCppSlotsAggregatedStatus,
	remainingTicksUntilRemoved int,
) (int, int) {
	self.RBMutex.Lock()
	defer self.RBMutex.Unlock()

	slotsIdleDiff := llamaCppSlotsAggregatedStatus.SlotsIdle - self.LlamaCppSlotsAggregatedStatus.SlotsIdle
	slotsProcessingDiff := llamaCppSlotsAggregatedStatus.SlotsProcessing - self.LlamaCppSlotsAggregatedStatus.SlotsProcessing

	self.LastUpdate = lastUpdate
	self.LlamaCppSlotsAggregatedStatus = llamaCppSlotsAggregatedStatus
	self.RemainingTicksUntilRemoved = remainingTicksUntilRemoved
	self.TotalUpdates += 1

	return slotsIdleDiff, slotsProcessingDiff
}
