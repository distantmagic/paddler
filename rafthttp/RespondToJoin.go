package rafthttp

import (
	"fmt"

	"github.com/distantmagic/paddler/raftstore"
	"github.com/valyala/fasthttp"
)

type RespondToJoin struct {
	RaftClusterController *raftstore.RaftClusterController
}

func (self *RespondToJoin) CreateResponse(ctx *fasthttp.RequestCtx) {
	configFuture := self.RaftClusterController.Raft.GetConfiguration()

	if err := configFuture.Error(); err != nil {
		ctx.Error(err.Error(), fasthttp.StatusInternalServerError)

		return
	}

	for _, srv := range configFuture.Configuration().Servers {
		fmt.Fprintf(
			ctx,
			"SRV: %+v\n",
			srv,
		)
	}

	fmt.Fprintf(
		ctx,
		"Hello, world! Requested path is %q",
		ctx.Path(),
	)
}
