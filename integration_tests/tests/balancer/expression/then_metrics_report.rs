use core::panic;

use anyhow::Result;
use cucumber::gherkin::Step;
use cucumber::then;

use crate::paddler_world::PaddlerWorld;

async fn fetch_metrics(statsd_port: u16, metric_name: String) -> Result<String> {
    let response = reqwest::get(format!(
        "http://localhost:{statsd_port}/metrics?query={metric_name}"
    ))
    .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Metric check failed: Expected status 200, got {}",
            response.status()
        ));
    }

    let body = response
        .text()
        .await?
        .replace(&format!("{metric_name} "), "");

    Ok(body)
}

#[then(expr = "metrics report:")]
pub async fn then_metrics_report(_world: &mut PaddlerWorld, step: &Step) -> Result<()> {
    if let Some(table) = step.table.as_ref() {
        for row in &table.rows {
            let metric_name = row[0].clone();
            let value = fetch_metrics(9102, format!("paddler_{metric_name}")).await?;
            assert_eq!(row[1], value.trim());
        }
    };

    Ok(())
}
