use std::collections::HashMap;
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
    let mut error_counts: HashMap<String, usize> = HashMap::new();

    while attempts < max_attempts {
        if let Err(err) = operation().await {
            *error_counts.entry(err.to_string()).or_insert(0) += 1;
        } else {
            return Ok(());
        }

        attempts += 1;

        sleep(delay).await;
    }

    let errors_combined = error_counts
        .iter()
        .map(|(error, count)| {
            if *count > 1 {
                format!("{error} ({count}x)")
            } else {
                error.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n  - ");

    Err(anyhow!(
        "{error_message} after {max_attempts} attempts.\nErrors:\n  - {errors_combined}\n\n"
    ))
}
