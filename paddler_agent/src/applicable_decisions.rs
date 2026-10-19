use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicableDecisions {
    Disabled,
    Enabled { pointer_head_path: PathBuf },
}
