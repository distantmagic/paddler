package raftstore

import (
	"github.com/distantmagic/paddler/netcfg"
	"github.com/hashicorp/go-hclog"
	"github.com/hashicorp/raft"
)

type RaftClusterController struct {
	Logger               hclog.Logger
	Raft                 *raft.Raft
	RaftConfiguration    *RaftConfiguration
	RaftNetworkTransport *raft.NetworkTransport
}

func (self *RaftClusterController) BootstrapCluster() raft.Future {
	self.Logger.Debug("bootstrap_cluster")

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

func (self *RaftClusterController) AddVoter(
	nodeId string,
	raftLeaderAddr *netcfg.HttpAddressConfiguration,
) error {
	serverNodeId := raft.ServerID(nodeId)
	raftLeaderServerAddr := raft.ServerAddress(
		raftLeaderAddr.GetHostWithPort(),
	)

	err := self.RemoveServerIfExists(serverNodeId, raftLeaderServerAddr)

	if err != nil {
		return err
	}

	addVoterFuture := self.Raft.AddVoter(
		serverNodeId,
		raftLeaderServerAddr,
		0,
		self.RaftConfiguration.Timeout,
	)

	if err := addVoterFuture.Error(); err != nil {
		return err
	}

	return nil
}

func (self *RaftClusterController) RemoveServerIfExists(
	serverNodeId raft.ServerID,
	raftLeaderServerAddr raft.ServerAddress,
) error {
	configFuture := self.Raft.GetConfiguration()

	if err := configFuture.Error(); err != nil {
		return err
	}

	for _, srv := range configFuture.Configuration().Servers {
		if srv.ID == serverNodeId || srv.Address == raftLeaderServerAddr {
			removeServerFuture := self.Raft.RemoveServer(srv.ID, 0, 0)

			if err := removeServerFuture.Error(); err != nil {
				return err
			}
		}
	}

	return nil
}
