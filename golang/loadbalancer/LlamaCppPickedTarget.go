package loadbalancer

import "container/list"

type LlamaCppPickedTarget struct {
	Element        *list.Element
	LlamaCppTarget *LlamaCppTarget
}
