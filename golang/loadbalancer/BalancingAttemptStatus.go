package loadbalancer

type BalancingAttemptStatus struct {
	Error          error
	LlamaCppTarget *LlamaCppTarget
}
