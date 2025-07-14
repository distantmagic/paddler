pub trait ProducesSnapshot {
    type Snapshot;

    fn make_snapshot(&self) -> Self::Snapshot;
}
