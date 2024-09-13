package loadbalancer

func LlamaCppTargetBalancingCompare(a, b *LlamaCppTarget) int {
	if a.HasLessSlotsThan(b) {
		return 1
	}

	if b.HasLessSlotsThan(a) {
		return -1
	}

	return 0
}
