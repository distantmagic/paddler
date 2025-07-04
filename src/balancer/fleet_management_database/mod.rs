mod file;
mod memory;

pub use self::file::File;
pub use self::memory::Memory;

pub trait FleetManagementDatabase {}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tempfile::NamedTempFile;

    use super::*;

    async fn subtest_store_desired_state<TDatabase: FleetManagementDatabase>(
        db: &TDatabase,
    ) -> Result<()> {
        // Placeholder for actual test logic
        Ok(())
    }

    #[tokio::test]
    async fn test_file_database() -> Result<()> {
        let tempfile = NamedTempFile::new()?;
        let db = File::new(tempfile.path().to_path_buf());

        subtest_store_desired_state(&db).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_memory_database() -> Result<()> {
        let db = Memory::new();

        subtest_store_desired_state(&db).await?;

        Ok(())
    }
}
