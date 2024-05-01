package main

import (
	"log"
	"os"
	"time"

	"github.com/distantmagic/paddler/netcfg"
	"github.com/distantmagic/paddler/rafthttp"
	"github.com/distantmagic/paddler/raftstore"
)

func main() {
	logger := log.New(os.Stderr, "[paddler] ", log.LstdFlags)

	raftClusterControllerBuilder := &raftstore.RaftClusterControllerBuilder{
		FiniteStateMachine: &raftstore.FiniteStateMachine{},
		Logger:             logger,
		RaftConfiguration: &raftstore.RaftConfiguration{
			HttpAddress: &netcfg.HttpAddressConfiguration{
				Host:   "127.0.0.1",
				Port:   11000,
				Scheme: "http",
			},
			LocalClusterNodeId:  "node1",
			LogDatabaseFile:     "raft.db",
			MaxPool:             5,
			RetainSnapshotCount: 2,
			SnapshotDirectory:   ".",
			Timeout:             time.Second * 5,
		},
	}

	raftClustercontroller, err := raftClusterControllerBuilder.BuildRaftClusterController()

	if err != nil {
		panic(err)
	}

	bootstrapClusterFuture := raftClustercontroller.BootstrapCluster()

	if err := bootstrapClusterFuture.Error(); err != nil {
		panic(err)
	}

	server := &rafthttp.Server{
		Logger: logger,
		RespondToJoin: &rafthttp.RespondToJoin{
			RaftClusterController: raftClustercontroller,
		},
	}

	serverEventsChannel := make(chan rafthttp.ServerEvent)

	go server.Serve(serverEventsChannel)

	for serverEvent := range serverEventsChannel {
		log.Println(serverEvent)
	}
}
