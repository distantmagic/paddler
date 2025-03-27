use std::time::Duration;

use actix_web::web::Bytes;
use cucumber::{given, then, when, World};
use httpmock::{Method::GET, MockServer};
use serde_json::json;
use tokio::sync::broadcast::{channel, Receiver};

use crate::{
    agent::monitoring_service::MonitoringService,
    balancer::status_update::StatusUpdate,
    errors::{app_error::AppError, result::Result},
    llamacpp::llamacpp_client::LlamacppClient,
};

#[derive(Default)]
struct MockLlamacpp(Option<MockServer>);

impl std::fmt::Debug for MockLlamacpp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MockServer")
    }
}

#[derive(Debug, Default, cucumber::World)]
struct MonitoringServicetWorld {
    pub mock: MockLlamacpp,
    pub monitoring_service: Option<MonitoringService>,
    pub response: Option<StatusUpdate>,
    pub error: Option<AppError>,
    pub report: Option<Receiver<Bytes>>,
}

#[given(regex = r"llamacpp 1 server is running")]
async fn setup_llamacpp_server(world: &mut MonitoringServicetWorld) -> Result<()> {
    let mock_server = MockServer::start();
    let (status_update_tx, status_update_rx) = channel::<Bytes>(1);

    world.monitoring_service = Some(MonitoringService {
        external_llamacpp_addr: *mock_server.address(),
        llamacpp_client: LlamacppClient::new(*mock_server.address(), None)?,
        monitoring_interval: Duration::from_secs(1),
        name: Some("Llama.cpp 1".to_string()),
        status_update_tx,
    });
    world.report = Some(status_update_rx);

    world.mock.0 = Some(mock_server);

    Ok(())
}

#[when(regex = r"monitoring service fetches slots endpoint")]
async fn fetch_slots_status(world: &mut MonitoringServicetWorld) -> Result<()> {
    let mock_server = world.mock.0.as_ref().unwrap();
    let _mock = mock_server.mock(|when, then| {
        when.method(GET).path("/slots");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!([
                {
                    "id": 0,
                    "is_processing": false,
                    "prompt": "",
                }
            ]));
    });

    if let Some(monitoring_service) = &world.monitoring_service {
        world.response = Some(monitoring_service.fetch_status().await?);
    };

    world.error = None;

    Ok(())
}

#[then(regex = r"monitoring service must receive a successful response")]
async fn receive_successful_response(world: &mut MonitoringServicetWorld) {
    if let Some(response) = world.response.as_ref() {
        assert!(world.error.is_none());
        assert_eq!(response.agent_name, Some("Llama.cpp 1".to_string()));
        assert_eq!(response.error, None);
        assert_eq!(
            response.external_llamacpp_addr,
            *world.mock.0.as_ref().unwrap().address()
        );
        assert_eq!(response.is_authorized, Some(true));
        assert_eq!(response.is_slots_endpoint_enabled, Some(true));
        assert_eq!(response.processing_slots_count, 0);
    }
}

#[when(regex = r"monitoring server reports status")]
async fn report_status(world: &mut MonitoringServicetWorld) -> Result<()> {
    if let Some(monitoring_service) = &world.monitoring_service {
        let status = world.response.as_ref().unwrap().to_owned();

        let _ = monitoring_service.report_status(status).await?;
    };

    world.error = None;

    Ok(())
}

#[then(regex = r"monitoring service must receive a successful report response")]
async fn receive_successful_response_from_report(
    world: &mut MonitoringServicetWorld,
) -> Result<()> {
    if let Some(status_receiver) = world.report.as_mut() {
        let status = status_receiver.recv().await?;
        let status_update = serde_json::from_slice::<StatusUpdate>(&status)?;

        assert!(world.error.is_none());
        assert_eq!(status_update.agent_name, Some("Llama.cpp 1".to_string()));
        assert_eq!(status_update.error, None);
        assert_eq!(
            status_update.external_llamacpp_addr,
            *world.mock.0.as_ref().unwrap().address()
        );
        assert_eq!(status_update.is_authorized, Some(true));
        assert_eq!(status_update.is_slots_endpoint_enabled, Some(true));
        assert_eq!(status_update.processing_slots_count, 0);
    }

    Ok(())
}

#[tokio::test]
async fn run_cucumber_tests() {
    MonitoringServicetWorld::run("src/tests/features/monitoring_service.feature").await;
}
