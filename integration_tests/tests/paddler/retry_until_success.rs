use std::collections::HashSet;
use std::future::Future;
use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use tokio::time::sleep;

pub async fn retry_until_success<FOperation, Fut>(
    operation: FOperation,
    max_attempts: usize,
    delay: Duration,
    error_message: String,
) -> Result<()>
where
    FOperation: Fn() -> Fut,
    Fut: Future<Output = Result<()>>,
{
    let mut attempts = 0;
    let mut errors: HashSet<String> = HashSet::new();

    while attempts < max_attempts {
        if let Err(err) = operation().await {
            errors.insert(err.to_string());
        } else {
            return Ok(());
        }

        attempts += 1;

        sleep(delay).await;
    }

    let errors_combined = errors.iter().cloned().collect::<Vec<_>>().join("\n  - ");

    Err(anyhow!(
        "{error_message} after {max_attempts} attempts.\nErrors:\n  - {errors_combined}\n\n"
    ))
}
