package main

import (
	"flag"
	"log"
	"os"
	"time"

	"github.com/distantmagic/paddler/netcfg"
	"github.com/distantmagic/paddler/rafthttp"
	"github.com/distantmagic/paddler/raftstore"
)

var (
	FlagBootstrap          = flag.Bool("bootstrap", false, "Bootstrap the cluster")
	FlagHost               = flag.String("host", "127.0.0.1", "Host to bind to")
	FlagLocalClusterNodeId = flag.String("node-id", "node1", "Local cluster node id")
	FlagLogDatabaseFile    = flag.String("log-db-file", "log.db", "Log database file")
	FlagMaxPool            = flag.Int("max-pool", 5, "How many connections to pool")
	FlagPort               = flag.Uint("port", 11000, "Port to bind to")
	FlagRetainSnapsotCount = flag.Int("retain-snapshot-count", 2, "How many snapshots to retain")
	FlagScheme             = flag.String("scheme", "http", "Scheme to use")
	FlagSnapsotDirectory   = flag.String("snapshot-directory", ".", "Directory to retain snapshots")
	FlagStableDatabaseFile = flag.String("stable-db-file", "stable.db", "Stable database file")
	FlagTimeout            = flag.Int("timeout", 5000, "(Miliseconds) Timeout is used to apply I/O deadlines")
)

func main() {
	logger := log.New(os.Stderr, "[paddler] ", log.LstdFlags)

	raftClusterControllerBuilder := &raftstore.RaftClusterControllerBuilder{
		FiniteStateMachine: &raftstore.FiniteStateMachine{},
		Logger:             logger,
		RaftConfiguration: &raftstore.RaftConfiguration{
			HttpAddress: &netcfg.HttpAddressConfiguration{
				Host:   *FlagHost,
				Port:   *FlagPort,
				Scheme: *FlagScheme,
			},
			LocalClusterNodeId:  *FlagLocalClusterNodeId,
			LogDatabaseFile:     *FlagLogDatabaseFile,
			StableDatabaseFile:  *FlagStableDatabaseFile,
			MaxPool:             *FlagMaxPool,
			RetainSnapshotCount: *FlagRetainSnapsotCount,
			SnapshotDirectory:   *FlagSnapsotDirectory,
			Timeout:             time.Millisecond * time.Duration(*FlagTimeout),
		},
	}

	raftClustercontroller, err := raftClusterControllerBuilder.BuildRaftClusterController()

	if err != nil {
		panic(err)
	}

	if *FlagBootstrap {
		bootstrapClusterFuture := raftClustercontroller.BootstrapCluster()

		if err := bootstrapClusterFuture.Error(); err != nil {
			panic(err)
		}
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
