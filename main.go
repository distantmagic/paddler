package main

import (
	"flag"
	"time"

	"github.com/distantmagic/paddler/netcfg"
	"github.com/distantmagic/paddler/paddlerhttp"
	"github.com/distantmagic/paddler/raftstore"
	"github.com/hashicorp/go-hclog"
)

var (
	FlagBootstrap          = flag.Bool("bootstrap", false, "Bootstrap the cluster")
	FlagLocalClusterNodeId = flag.String("node-id", "node1", "Local cluster node id")
	FlagLogDatabaseFile    = flag.String("log-db-file", "log.db", "Log database file")
	FlagMaxPool            = flag.Int("max-pool", 5, "How many connections to pool")
	FlagPaddlerHost        = flag.String("paddler-host", "127.0.0.1", "Paddler host to bind to")
	FlagPaddlerPort        = flag.Uint("paddler-port", 8080, "Paddler port to bind to")
	FlagPaddlerScheme      = flag.String("paddler-scheme", "http", "Paddler scheme to use")
	FlagJoin               = flag.Bool("join", false, "Join the cluster")
	FlagRaftHost           = flag.String("raft-host", "127.0.0.1", "Raft host to bind to")
	FlagRaftPort           = flag.Uint("raft-port", 11000, "Raft port to bind to")
	FlagRaftScheme         = flag.String("raft-scheme", "http", "Raft scheme to use")
	FlagRetainSnapsotCount = flag.Int("retain-snapshot-count", 2, "How many snapshots to retain")
	FlagSnapsotDirectory   = flag.String("snapshot-directory", ".", "Directory to retain snapshots")
	FlagStableDatabaseFile = flag.String("stable-db-file", "stable.db", "Stable database file")
	FlagTimeout            = flag.Int("timeout", 5000, "(Miliseconds) Timeout is used to apply I/O deadlines")
)

func main() {
	flag.Parse()

	logger := hclog.New(&hclog.LoggerOptions{
		Name:  "paddler",
		Level: hclog.Debug,
	})

	raftClusterControllerBuilder := &raftstore.RaftClusterControllerBuilder{
		FiniteStateMachine: &raftstore.FiniteStateMachine{},
		Logger:             logger.Named("raftstore"),
		RaftConfiguration: &raftstore.RaftConfiguration{
			HttpAddress: &netcfg.HttpAddressConfiguration{
				Host:   *FlagRaftHost,
				Port:   *FlagRaftPort,
				Scheme: *FlagRaftScheme,
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

	if *FlagJoin {
		// err := raftClustercontroller.AddVoter(
		// 	*FlagLocalClusterNodeId,
		// 	&netcfg.HttpAddressConfiguration{
		// 		Host:   *FlagJoinHost,
		// 		Port:   *FlagJoinPort,
		// 		Scheme: *FlagJoinScheme,
		// 	},
		// )

		// if err != nil {
		// 	panic(err)
		// }
	}

	server := &paddlerhttp.Server{
		HttpAddress: &netcfg.HttpAddressConfiguration{
			Host:   *FlagPaddlerHost,
			Port:   *FlagPaddlerPort,
			Scheme: *FlagPaddlerScheme,
		},
		Logger: logger.Named("paddlerhttp.Server"),
		RespondToList: &paddlerhttp.RespondToList{
			RaftClusterController: raftClustercontroller,
		},
	}

	serverEventsChannel := make(chan paddlerhttp.ServerEvent)

	go server.Serve(serverEventsChannel)

	for serverEvent := range serverEventsChannel {
		logger.Info("server event", serverEvent)
	}
}
