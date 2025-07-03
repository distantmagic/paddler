use std::path::PathBuf;

use super::FleetManagementDatabase;

pub struct Lmdb {
    base_path: PathBuf,
}

impl Lmdb {
    pub fn new(base_path: PathBuf) -> Self {
        Lmdb {
            base_path,
        }
    }
}

impl FleetManagementDatabase for Lmdb {}
