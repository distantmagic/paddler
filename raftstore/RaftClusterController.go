package raftstore

import (
	"log"

	"github.com/hashicorp/raft"
)

type RaftClusterController struct {
	Logger               *log.Logger
	Raft                 *raft.Raft
	RaftConfiguration    *RaftConfiguration
	RaftNetworkTransport *raft.NetworkTransport
}

func (self *RaftClusterController) AddVoter(nodeId, joiningNodeAddr string) error {
	self.Logger.Printf("raftstore.RaftClusterController.AddVoter(%s, %s)", nodeId, joiningNodeAddr)

	serverNodeId := raft.ServerID(nodeId)
	joiningNodeServerAddr := raft.ServerAddress(joiningNodeAddr)

	err := self.RemoveServerIfExists(serverNodeId, joiningNodeServerAddr)

	if err != nil {
		return err
	}

	addVoterFuture := self.Raft.AddVoter(
		serverNodeId,
		joiningNodeServerAddr,
		0,
		self.RaftConfiguration.Timeout,
	)

	if err := addVoterFuture.Error(); err != nil {
		return err
	}

	return nil
}

func (self *RaftClusterController) BootstrapCluster() raft.Future {
	configuration := raft.Configuration{
		Servers: []raft.Server{
			{
				ID:      raft.ServerID(self.RaftConfiguration.LocalClusterNodeId),
				Address: self.RaftNetworkTransport.LocalAddr(),
			},
		},
	}

	return self.Raft.BootstrapCluster(configuration)
}

func (self *RaftClusterController) RemoveServerIfExists(
	serverNodeId raft.ServerID,
	joiningNodeServerAddr raft.ServerAddress,
) error {
	configFuture := self.Raft.GetConfiguration()

	if err := configFuture.Error(); err != nil {
		return err
	}

	for _, srv := range configFuture.Configuration().Servers {
		if srv.ID == serverNodeId || srv.Address == joiningNodeServerAddr {
			removeServerFuture := self.Raft.RemoveServer(srv.ID, 0, 0)

			if err := removeServerFuture.Error(); err != nil {
				return err
			}
		}
	}

	return nil
}
