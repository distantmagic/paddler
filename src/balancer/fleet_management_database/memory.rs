use super::FleetManagementDatabase;

pub struct Memory {}

impl Memory {
    pub fn new() -> Self {
        Memory {}
    }
}

impl FleetManagementDatabase for Memory {}
