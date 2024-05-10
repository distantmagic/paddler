package loadbalancer

func LlamaCppTargetComparator(a, b *LlamaCppTarget) int {
	if a.Less(b) {
		return -1
	}

	if b.Less(a) {
		return 1
	}

	return 0
}
