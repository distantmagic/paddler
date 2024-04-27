package llamacpp

import (
	"fmt"
	"net/url"
)

type LlamaCppConfiguration struct {
	Host   string
	Port   uint
	Scheme string
}

func (self *LlamaCppConfiguration) BuildUrlWithPath(path string) *url.URL {
	return &url.URL{
		Scheme: self.Scheme,
		Host:   fmt.Sprintf("%s:%d", self.Host, self.Port),
		Path:   path,
	}
}
