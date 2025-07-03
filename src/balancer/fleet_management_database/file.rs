use std::path::PathBuf;

use super::FleetManagementDatabase;

pub struct File {
    path: PathBuf,
}

impl File {
    pub fn new(path: PathBuf) -> Self {
        File {
            path,
        }
    }
}

impl FleetManagementDatabase for File {}
