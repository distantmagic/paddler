package netcfg

import (
	"fmt"
	"net/url"
)

type HttpAddressConfiguration struct {
	Host   string
	Port   uint
	Scheme string
}

func (self *HttpAddressConfiguration) BuildUrlWithPath(path string) *url.URL {
	return &url.URL{
		Scheme: self.Scheme,
		Host:   self.GetHostWithPort(),
		Path:   path,
	}
}

func (self *HttpAddressConfiguration) GetHostWithPort() string {
	return fmt.Sprintf("%s:%d", self.Host, self.Port)
}
