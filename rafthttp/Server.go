package rafthttp

import (
	"log"

	"github.com/fasthttp/router"
	"github.com/valyala/fasthttp"
)

type Server struct {
	Logger        *log.Logger
	RespondToJoin *RespondToJoin
}

func (self *Server) Serve(serverEventsChannel chan ServerEvent) {
	defer close(serverEventsChannel)

	self.Logger.Println("rafthttp.Server.Serve()")

	routes := router.New()
	routes.GET("/join", self.RespondToJoin.CreateResponse)

	fasthttp.ListenAndServe(":8080", routes.Handler)
}
