package llamacpp

type LlamaCppCompletionRequest struct {
	JsonSchema any    `json:"json_schema"`
	NPredict   int    `json:"n_predict"`
	Prompt     string `json:"prompt"`
	Stream     bool   `json:"stream"`
}
