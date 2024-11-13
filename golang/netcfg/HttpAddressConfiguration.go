package netcfg

import (
	"fmt"
	"net/url"
)

type HttpAddressConfiguration struct {
	Host   string `json:"host"`
	Port   uint   `json:"port"`
	Scheme string `json:"scheme"`
}

func (self *HttpAddressConfiguration) BuildUrlWithPath(path string) *url.URL {
	return &url.URL{
		Scheme: self.Scheme,
		Host:   self.GetHostWithPort(),
		Path:   path,
	}
}

func (self *HttpAddressConfiguration) GetBaseUrl() *url.URL {
	return self.BuildUrlWithPath("")
}

func (self *HttpAddressConfiguration) GetHostWithPort() string {
	return fmt.Sprintf("%s:%d", self.Host, self.Port)
}

func (self *HttpAddressConfiguration) IsSameAs(other *HttpAddressConfiguration) bool {
	return self.Host == other.Host &&
		self.Port == other.Port &&
		self.Scheme == other.Scheme
}

func (self *HttpAddressConfiguration) String() string {
	return self.BuildUrlWithPath("").String()
}
