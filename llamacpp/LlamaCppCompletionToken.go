package llamacpp

type LlamaCppCompletionToken struct {
	Content string `json:"content"`
	Error error `json:"-"`
	IsLast bool `json:"stop"`
	SlotId uint `json:"id_slot"`
}
