package raftstore

import "github.com/hashicorp/raft"

// FSMSnapshot is returned by an FSM in response to a Snapshot
// It must be safe to invoke FSMSnapshot methods with concurrent
// calls to Apply.
type FiniteStateMachineSnapshot struct {
}

// Persist should dump all necessary state to the WriteCloser 'sink',
// and call sink.Close() when finished or call sink.Cancel() on error.
func (self *FiniteStateMachineSnapshot) Persist(sink raft.SnapshotSink) error {
	return nil
}

// Release is invoked when we are finished with the snapshot.
func (self *FiniteStateMachineSnapshot) Release() {
}
