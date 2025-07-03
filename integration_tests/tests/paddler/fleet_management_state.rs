use tempfile::TempDir;

#[derive(Debug)]
pub struct FleetManagementState {
    pub fleet_database_directory: TempDir,
}
