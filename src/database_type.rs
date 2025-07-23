use std::path::PathBuf;
use std::str::FromStr;

use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use url::Url;

#[derive(Clone)]
pub enum DatabaseType {
    File(PathBuf),
    Memory,
}

impl FromStr for DatabaseType {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(input)?;

        match url.scheme() {
            "file" => {
                let path = url.path();

                if url.host().is_some() {
                    return Err(anyhow!("URL must contain only absolute path"));
                }

                if path.is_empty() || path == "/" {
                    return Err(anyhow!(
                        "URL must specify a path: file:///path/to/directory"
                    ));
                }

                Ok(DatabaseType::File(PathBuf::from(path)))
            }
            "memory" => Ok(DatabaseType::Memory),
            scheme => Err(anyhow!("Unsupported scheme '{scheme}'")),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_memory_basic() {
        let result = DatabaseType::from_str("memory://").unwrap();
        assert!(matches!(result, DatabaseType::Memory));
    }

    #[test]
    fn test_lmdb_relative_path() {
        let result = DatabaseType::from_str("file://path/to/db");

        assert!(result.is_err());
    }

    #[test]
    fn test_lmdb_absolute_path() {
        let result = DatabaseType::from_str("file:///absolute/path").unwrap();
        match result {
            DatabaseType::File(path) => {
                assert_eq!(path, PathBuf::from("/absolute/path"));
            }
            _ => panic!("Expected File variant"),
        }
    }

    #[test]
    fn test_lmdb_empty_path_fails() {
        let result = DatabaseType::from_str("file://");

        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_scheme() {
        let result = DatabaseType::from_str("mysql://localhost/db");

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_url() {
        let result = DatabaseType::from_str("not-a-url");

        assert!(result.is_err());
    }
}
