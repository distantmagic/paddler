#[cfg(test)]
mod tests {
    use actix_web::web::Bytes;
    use httpmock::{Method::GET, MockServer};
    use serde_json::json;
    use tokio::sync::broadcast::channel;

    use crate::{
        agent::monitoring_service::MonitoringService, errors::result::Result,
        llamacpp::llamacpp_client::LlamacppClient,
    };

    #[tokio::test]
    async fn slots_are_unathorized() -> Result<()> {
        let mock_server = MockServer::start();
        let (status_update_tx, _status_update_rx) = channel::<Bytes>(1);

        let monitoring_service = MonitoringService::new(
            *mock_server.address(),
            LlamacppClient::new(*mock_server.address(), None)?,
            std::time::Duration::from_secs(1),
            Some("Llama.cpp 1".to_string()),
            status_update_tx,
        )?;

        let llamacpp = mock_server.mock(|when, then| {
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
        llamacpp.assert_async().await;

        Ok(())
    }

    #[tokio::test]
    async fn response_is_unimplemented() -> Result<()> {
        let mock_server = MockServer::start();
        let (status_update_tx, _status_update_rx) = channel::<Bytes>(1);

        let monitoring_service = MonitoringService::new(
            *mock_server.address(),
            LlamacppClient::new(*mock_server.address(), None)?,
            std::time::Duration::from_secs(1),
            Some("Llama.cpp 1".to_string()),
            status_update_tx,
        )?;

        let llamacpp = mock_server.mock(|when, then| {
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
        llamacpp.assert_async().await;

        Ok(())
    }

    #[tokio::test]
    async fn response_is_error() -> Result<()> {
        let mock_server = MockServer::start();
        let (status_update_tx, _status_update_rx) = channel::<Bytes>(1);

        let monitoring_service = MonitoringService::new(
            *mock_server.address(),
            LlamacppClient::new(*mock_server.address(), None)?,
            std::time::Duration::from_secs(1),
            Some("Llama.cpp 1".to_string()),
            status_update_tx,
        )?;

        let llamacpp = mock_server.mock(|when, then| {
            when.method(GET).path("/slots");
            then.status(99);
        });

        let status_update = monitoring_service.fetch_status().await?;

        assert!(status_update.error.is_some());
        llamacpp.assert_async().await;

        Ok(())
    }
}
