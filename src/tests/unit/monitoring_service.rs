use std::time::Duration;

use actix_web::web::Bytes;
use httpmock::{Method::GET, MockServer};
use serde_json::json;
use tokio::sync::broadcast::channel;

use crate::{
    agent::monitoring_service::MonitoringService, balancer::status_update::StatusUpdate,
    errors::result::Result, llamacpp::llamacpp_client::LlamacppClient,
};

#[tokio::test]
async fn slots_are_authorized() -> Result<()> {
    let mock_server = MockServer::start();
    let (status_update_tx, _status_update_rx) = channel::<Bytes>(1);

    let monitoring_service = MonitoringService::new(
        *mock_server.address(),
        LlamacppClient::new(*mock_server.address(), None)?,
        Duration::from_secs(1),
        Some("Llama.cpp 1".to_string()),
        status_update_tx,
    )?;

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

    let response = monitoring_service.fetch_status().await?;

    assert_eq!(response.agent_name, Some("Llama.cpp 1".to_string()));
    assert_eq!(response.error, None);
    assert_eq!(response.external_llamacpp_addr, *mock_server.address());
    assert_eq!(response.is_authorized, Some(true));
    assert_eq!(response.is_slots_endpoint_enabled, Some(true));
    assert_eq!(response.processing_slots_count, 0);

    Ok(())
}

#[tokio::test]
async fn slots_are_unathorized() -> Result<()> {
    let mock_server = MockServer::start();
    let (status_update_tx, _status_update_rx) = channel::<Bytes>(1);

    let monitoring_service = MonitoringService::new(
        *mock_server.address(),
        LlamacppClient::new(*mock_server.address(), None)?,
        Duration::from_secs(1),
        Some("Llama.cpp 1".to_string()),
        status_update_tx,
    )?;

    let _mock = mock_server.mock(|when, then| {
        when.method(GET).path("/slots");
        then.status(401)
            .header("content-type", "application/json")
            .json_body(json!([
                {
                    "id": 0,
                    "is_processing": false,
                    "prompt": "",
                }
            ]));
    });

    let response = monitoring_service.fetch_status().await?;

    assert!(!response.is_authorized.unwrap());
    assert!(response.is_slots_endpoint_enabled.is_none());
    assert!(response.slots.is_empty());

    Ok(())
}

#[tokio::test]
async fn response_is_unimplemented() -> Result<()> {
    let mock_server = MockServer::start();
    let (status_update_tx, _status_update_rx) = channel::<Bytes>(1);

    let monitoring_service = MonitoringService::new(
        *mock_server.address(),
        LlamacppClient::new(*mock_server.address(), None)?,
        Duration::from_secs(1),
        Some("Llama.cpp 1".to_string()),
        status_update_tx,
    )?;

    let _mock = mock_server.mock(|when, then| {
        when.method(GET).path("/slots");
        then.status(501)
            .header("content-type", "application/json")
            .json_body(json!([
                {
                    "id": 0,
                    "is_processing": false,
                    "prompt": "",
                }
            ]));
    });

    let response = monitoring_service.fetch_status().await?;

    assert!(response.is_authorized.is_none());
    assert!(!response.is_slots_endpoint_enabled.unwrap());
    assert!(response.slots.is_empty());

    Ok(())
}

#[tokio::test]
async fn response_is_error() -> Result<()> {
    let mock_server = MockServer::start();
    let (status_update_tx, _status_update_rx) = channel::<Bytes>(1);

    let monitoring_service = MonitoringService::new(
        *mock_server.address(),
        LlamacppClient::new(*mock_server.address(), None)?,
        Duration::from_secs(1),
        Some("Llama.cpp 1".to_string()),
        status_update_tx,
    )?;

    let _mock = mock_server.mock(|when, then| {
        when.method(GET).path("/slots");
        then.status(99);
    });

    let status_update = monitoring_service.fetch_status().await?;

    assert!(status_update.error.is_some());

    Ok(())
}

#[tokio::test]
async fn report_is_successful() -> Result<()> {
    let mock_server = MockServer::start();
    let (status_update_tx, mut status_update_rx) = channel::<Bytes>(1);

    let monitoring_service = MonitoringService::new(
        *mock_server.address(),
        LlamacppClient::new(*mock_server.address(), None)?,
        Duration::from_secs(1),
        Some("Llama.cpp 1".to_string()),
        status_update_tx,
    )?;

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

    let response = monitoring_service.fetch_status().await?;

    monitoring_service.report_status(response).await?;

    let status = status_update_rx.recv().await?;
    let status_update = serde_json::from_slice::<StatusUpdate>(&status)?;

    assert_eq!(status_update.agent_name, Some("Llama.cpp 1".to_string()));
    assert_eq!(status_update.error, None);
    assert_eq!(
        status_update.external_llamacpp_addr,
        monitoring_service.external_llamacpp_addr
    );
    assert_eq!(status_update.is_authorized, Some(true));
    assert_eq!(status_update.is_slots_endpoint_enabled, Some(true));
    assert_eq!(status_update.processing_slots_count, 0);

    Ok(())
}

fn unsafe_code() {
    use std::ptr;

    unsafe fn raw_pointer_demo() {
        let mut num = 10;
        let raw_ptr = &mut num as *mut i32;

        *raw_ptr += 5;
        println!("Value after unsafe modification: {}", *raw_ptr);
    }
}
