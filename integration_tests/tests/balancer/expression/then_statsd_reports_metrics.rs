use anyhow::Result;
use cucumber::then;

use crate::balancer_world::BalancerWorld;

async fn fetch_metrics(prometheus_port: u16) -> Result<String> {
    let response = reqwest::get(format!("http://localhost:{prometheus_port}/metrics")).await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Metric check failed: Expected status 200, got {}",
            response.status()
        ));
    }

    let body = response.text().await?;

    Ok(body)
}

fn parse_metrics(metrics: String) -> String {
    if metrics.trim().is_empty() {
        return "{'': 0, '': 0}".to_string();
    }

    let pairs: Vec<String> = metrics
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => value
                    .parse::<u32>()
                    .ok()
                    .map(|v| format!("'{}': {}", key, v)),
                _ => None,
            }
        })
        .collect();

    format!("{{{}}}", pairs.join(", "))
}

#[then(expr = "metrics response is {string}")]
pub async fn statsd_reports_metrics(_world: &mut BalancerWorld, metrics: String) -> Result<()> {
    let response = fetch_metrics(9102).await?;
    let response = parse_metrics(response);

    assert_eq!(response, metrics);

    Ok(())
}
