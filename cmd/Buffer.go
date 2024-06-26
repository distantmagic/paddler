package cmd

import (
	"github.com/hashicorp/go-hclog"
	"github.com/urfave/cli/v2"
)

type Buffer struct {
	Logger                        hclog.Logger
}

func (self *Buffer) Action(cliContext *cli.Context) error {
	self.Logger.Info("TODO")

	return nil
}
