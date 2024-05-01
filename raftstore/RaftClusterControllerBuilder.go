package raftstore

import (
	"log"
	"net"
	"os"

	"github.com/hashicorp/raft"
	raftboltdb "github.com/hashicorp/raft-boltdb"
)

type RaftClusterControllerBuilder struct {
	FiniteStateMachine *FiniteStateMachine
	Logger             *log.Logger
	RaftConfiguration  *RaftConfiguration
}

func (self *RaftClusterControllerBuilder) BuildFileSnapshotStore() (*raft.FileSnapshotStore, error) {
	return raft.NewFileSnapshotStore(
		self.RaftConfiguration.SnapshotDirectory,
		self.RaftConfiguration.RetainSnapshotCount,
		os.Stderr,
	)
}

func (self *RaftClusterControllerBuilder) BuildRaft(
	transport *raft.NetworkTransport,
) (*raft.Raft, error) {
	raftStore, err := self.BuildRaftStore()

	if err != nil {
		return nil, err
	}

	snapshots, err := self.BuildFileSnapshotStore()

	if err != nil {
		return nil, err
	}

	config := raft.DefaultConfig()
	config.LocalID = raft.ServerID(self.RaftConfiguration.LocalClusterNodeId)

	return raft.NewRaft(
		config,
		self.FiniteStateMachine,
		raftStore,
		raftStore,
		snapshots,
		transport,
	)
}

func (self *RaftClusterControllerBuilder) BuildRaftClusterController() (*RaftClusterController, error) {
	transport, err := self.BuildRaftNetworkTransport()

	if err != nil {
		return nil, err
	}

	raft, err := self.BuildRaft(transport)

	if err != nil {
		return nil, err
	}

	raftClusterNode := &RaftClusterController{
		Logger:               self.Logger,
		Raft:                 raft,
		RaftConfiguration:    self.RaftConfiguration,
		RaftNetworkTransport: transport,
	}

	return raftClusterNode, nil
}

func (self *RaftClusterControllerBuilder) BuildRaftNetworkTransport() (*raft.NetworkTransport, error) {
	advertiseAddr, err := net.ResolveTCPAddr(
		"tcp",
		self.RaftConfiguration.HttpAddress.GetHostWithPort(),
	)

	if err != nil {
		return nil, err
	}

	return raft.NewTCPTransport(
		self.RaftConfiguration.HttpAddress.GetHostWithPort(),
		advertiseAddr,
		self.RaftConfiguration.MaxPool,
		self.RaftConfiguration.Timeout,
		os.Stderr,
	)
}

func (self *RaftClusterControllerBuilder) BuildRaftStore() (*raftboltdb.BoltStore, error) {
	return raftboltdb.New(raftboltdb.Options{
		Path: self.RaftConfiguration.LogDatabaseFile,
	})
}
