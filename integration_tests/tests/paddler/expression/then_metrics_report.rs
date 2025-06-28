use core::panic;
use std::collections::BTreeMap;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use anyhow::Result;
use cucumber::gherkin::Step;
use cucumber::then;
use reqwest::Response;
use tokio::time::sleep;

use crate::paddler_world::PaddlerWorld;

const MAX_ATTEMPTS: u64 = 30;

async fn fetch_metrics(statsd_port: u16) -> Result<Response> {
    let response = reqwest::get(format!("http://localhost:{statsd_port}/metrics")).await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Metric dump failed: Expected status 200, got {}",
            response.status()
        ));
    }

    Ok(response)
}

async fn get_metrics_map(
    metrics: String,
    last_test_udpate: SystemTime,
) -> Result<BTreeMap<String, u64>> {
    let mut metrcis_map: BTreeMap<String, u64> = serde_json::from_str(metrics.trim())?;

    let last_mock_update = metrcis_map
        .get("last_update")
        .expect("Last update not found");

    let last_mock_update = UNIX_EPOCH + Duration::from_millis(*last_mock_update);
    metrcis_map.remove("last_update");

    eprintln!("passed time? {:#?}", (last_mock_update > last_test_udpate));

    if !(last_mock_update > last_test_udpate) {
        return Err(anyhow::anyhow!("Statsd metrics were not updated"));
    }

    Ok(metrcis_map)
}

#[then(expr = "metrics report:")]
pub async fn then_metrics_report(world: &mut PaddlerWorld, step: &Step) -> Result<()> {
    let mut table_metric_fields = BTreeMap::new();

    if let Some(table) = step.table.as_ref() {
        for row in &table.rows {
            let value: u64 = row
                .get(1)
                .expect("No metric value found")
                .parse()
                .expect("Failed to parse number");
            table_metric_fields.insert(
                "paddler_".to_string() + row.get(0).expect("No metric name found"),
                value,
            );
        }
    };

    let mut attempts = 0;

    while attempts < MAX_ATTEMPTS {
        sleep(Duration::from_millis(100)).await;

        let text_body = fetch_metrics(9102).await?.text().await?;
        let statsd_last_udpate = world.statsd.last_update.expect("Last update not found");

        if let Ok(stastd_metric_fields) = get_metrics_map(text_body, statsd_last_udpate).await {
            assert_eq!(table_metric_fields, stastd_metric_fields);
            world.statsd.last_update = Some(SystemTime::now());

            return Ok(());
        }

        attempts += 1;
    }

    Err(anyhow::anyhow!(
        "Balancer state did not update after {} attempts",
        MAX_ATTEMPTS
    ))
}
