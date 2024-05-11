package cmd

import (
	"github.com/distantmagic/paddler/llamacpp"
	"github.com/distantmagic/paddler/reverseproxy"
	"github.com/hashicorp/go-hclog"
	"github.com/urfave/cli/v2"
)

type Agent struct {
	Logger                    hclog.Logger
	LlamaCppConfiguration     *llamacpp.LlamaCppConfiguration
	ReverseProxyConfiguration *reverseproxy.ReverseProxyConfiguration
}

func (self *Agent) Action(cliContext *cli.Context) error {
	return nil
}
