use tempfile::NamedTempFile;

#[derive(Debug)]
pub struct FleetManagementState {
    pub fleet_database_file: NamedTempFile,
}
