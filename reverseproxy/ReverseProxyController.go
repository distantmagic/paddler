package reverseproxy

import (
	"net/http"
	"net/http/httputil"

	"github.com/hashicorp/go-hclog"
)

type ReverseProxyController struct {
	Logger       hclog.Logger
	ReverseProxy *httputil.ReverseProxy
}

func (self *ReverseProxyController) ServeHTTP(writer http.ResponseWriter, request *http.Request) {
	self.Logger.Debug("forwarding", "path", request.URL.Path)
	self.ReverseProxy.ServeHTTP(writer, request)
}
