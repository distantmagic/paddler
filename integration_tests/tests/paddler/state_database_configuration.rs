use tempfile::NamedTempFile;

#[derive(Debug)]
pub struct StateDatabaseConfiguration {
    pub database_file: NamedTempFile,
}
