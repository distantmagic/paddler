package llamacpp

type LlamaCppHealthStatusCode string

const (
	Error           LlamaCppHealthStatusCode = "error"
	LoadingModel    LlamaCppHealthStatusCode = "loading model"
	NoSlotAvailable LlamaCppHealthStatusCode = "no slot available"
	Ok              LlamaCppHealthStatusCode = "ok"
)
