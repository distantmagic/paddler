package rafthttp

import (
	"fmt"
	"log"

	"github.com/valyala/fasthttp"
)

type Server struct {
	Foobar string
	Logger *log.Logger
}

func (self *Server) HandleFastHTTP(ctx *fasthttp.RequestCtx) {
	switch string(ctx.Path()) {
	case "/foobar":
		fmt.Fprintf(
			ctx,
			"Hello, world! Requested path is %q. Foobar is %q",
			ctx.Path(),
			self.Foobar,
		)
	default:
		ctx.Error("not found", fasthttp.StatusNotFound)
	}
}

func (self *Server) Serve(serverEventsChannel chan ServerEvent) {
	defer close(serverEventsChannel)

	self.Logger.Println("rafthttp.Server.Serve()")

	fasthttp.ListenAndServe(":8080", self.HandleFastHTTP)
}
