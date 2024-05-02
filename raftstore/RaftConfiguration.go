package raftstore

import (
	"time"

	"github.com/distantmagic/paddler/netcfg"
)

type RaftConfiguration struct {
	HttpAddress         *netcfg.HttpAddressConfiguration
	LocalClusterNodeId  string
	LogDatabaseFile     string
	MaxPool             int
	RetainSnapshotCount int
	SnapshotDirectory   string
	StableDatabaseFile  string
	Timeout             time.Duration
}
