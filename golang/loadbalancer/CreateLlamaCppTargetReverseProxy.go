package loadbalancer

import (
	"net/http"
	"net/http/httputil"

	"github.com/hashicorp/go-hclog"
)

func CreateLlamaCppTargetReverseProxy(
	logger hclog.Logger,
	loadBalancerConfiguration *LoadBalancerConfiguration,
	targetConfiguration *LlamaCppTargetConfiguration,
) *httputil.ReverseProxy {
	reverseProxy := httputil.NewSingleHostReverseProxy(
		targetConfiguration.LlamaCppConfiguration.HttpAddress.GetBaseUrl(),
	)

	reverseProxy.ErrorLog = logger.StandardLogger(&hclog.StandardLoggerOptions{
		InferLevels: true,
	})

	if !loadBalancerConfiguration.RewriteHostHeader {
		return reverseProxy
	}

	originalDirector := reverseProxy.Director

	reverseProxy.Director = func(req *http.Request) {
		originalDirector(req)

		req.Host = targetConfiguration.LlamaCppConfiguration.HttpAddress.GetHostWithPort()
	}

	return reverseProxy
}
