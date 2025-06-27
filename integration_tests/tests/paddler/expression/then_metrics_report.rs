use core::panic;
use std::collections::BTreeMap;
use std::io::Read;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Result;
use cucumber::gherkin::Step;
use cucumber::then;

use crate::paddler_world::PaddlerWorld;

async fn dump_metrics_into_file(statsd_port: u16) -> Result<()> {
    let response = reqwest::get(format!("http://localhost:{statsd_port}/metrics")).await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Metric dump failed: Expected status 200, got {}",
            response.status()
        ));
    }

    Ok(())
}

#[then(expr = "metrics report:")]
pub async fn then_metrics_report(world: &mut PaddlerWorld, step: &Step) -> Result<()> {
    // sleep(Duration::from_millis(2000));
    dump_metrics_into_file(9102).await?;

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

    let mut content = "".to_string();
    let log_file = world.statsd.log_file.as_mut().expect("No log file found");
    log_file.read_to_string(&mut content)?;

    let stastd_metric_fields: BTreeMap<String, u64> =
        serde_json::from_str(content.trim()).expect("Failed to parse JSON");

    assert_eq!(table_metric_fields, stastd_metric_fields);

    Ok(())
}
