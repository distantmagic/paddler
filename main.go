package main

import (
	"log"
	"os"

	"github.com/distantmagic/paddler/rafthttp"
)

func main() {
	server := &rafthttp.Server{
		Foobar: "foobar",
		Logger: log.New(os.Stderr, "[paddler] ", log.LstdFlags),
	}

	serverEventsChannel := make(chan rafthttp.ServerEvent)

	go server.Serve(serverEventsChannel)

	for serverEvent := range serverEventsChannel {
		log.Println(serverEvent)
	}
}
