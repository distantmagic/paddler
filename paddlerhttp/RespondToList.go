package paddlerhttp

import (
	"fmt"

	"github.com/distantmagic/paddler/raftstore"
	"github.com/valyala/fasthttp"
)

type RespondToList struct {
	RaftClusterController *raftstore.RaftClusterController
}

func (self *RespondToList) CreateResponse(ctx *fasthttp.RequestCtx) {
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
