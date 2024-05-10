package cmd

import (
	"github.com/hashicorp/go-hclog"
	"github.com/urfave/cli/v2"
)

type Agent struct {
	Logger hclog.Logger
}

func (self *Agent) Action(cliContext *cli.Context) error {
	return nil
}
