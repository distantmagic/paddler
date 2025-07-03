use std::path::PathBuf;
use std::str::FromStr;

use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;
use url::Url;

#[derive(Clone)]
pub enum FleetDatabaseType {
    Memory,
    Lmdb(PathBuf),
}

impl FromStr for FleetDatabaseType {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let url = Url::parse(input)?;

        match url.scheme() {
            "memory" => Ok(FleetDatabaseType::Memory),
            "lmdb" => {
                let path = url.path();

                if url.host().is_some() {
                    return Err(anyhow!("LMDB URL must contain only absolute path"));
                }

                if path.is_empty() {
                    return Err(anyhow!(
                        "LMDB URL must specify a path: lmdb:///path/to/directory"
                    ));
                }

                Ok(FleetDatabaseType::Lmdb(PathBuf::from(path)))
            }
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
        let result = FleetDatabaseType::from_str("memory://").unwrap();
        matches!(result, FleetDatabaseType::Memory);
    }

    #[test]
    fn test_lmdb_relative_path() {
        let result = FleetDatabaseType::from_str("lmdb://path/to/db");

        assert!(result.is_err());
    }

    #[test]
    fn test_lmdb_absolute_path() {
        let result = FleetDatabaseType::from_str("lmdb:///absolute/path").unwrap();
        match result {
            FleetDatabaseType::Lmdb(path) => {
                assert_eq!(path, PathBuf::from("/absolute/path"));
            }
            _ => panic!("Expected Lmdb variant"),
        }
    }

    #[test]
    fn test_lmdb_empty_path_fails() {
        let result = FleetDatabaseType::from_str("lmdb://");

        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_scheme() {
        let result = FleetDatabaseType::from_str("mysql://localhost/db");

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_url() {
        let result = FleetDatabaseType::from_str("not-a-url");

        assert!(result.is_err());
    }
}
