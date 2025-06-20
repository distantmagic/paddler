use std::net::SocketAddr;
use std::time::Duration;

use reqwest::header;
use url::Url;

use crate::errors::result::Result;
use crate::llamacpp::slots_response::SlotsResponse;
use crate::llamacpp::slot::Slot;
use crate::llamacpp::models_response::ModelsResponse;

pub struct LlamacppClient {
    client: reqwest::Client,
    slots_endpoint_url: String,
    models_endpoint_url: String,
}

impl LlamacppClient {
    pub fn new(addr: SocketAddr, api_key: Option<String>) -> Result<Self> {
        let mut builder = reqwest::Client::builder().timeout(Duration::from_secs(3));
        let mut headers = header::HeaderMap::new();

        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );

        if let Some(api_key_value) = api_key {
            let mut auth_value = header::HeaderValue::from_str(&format!("Bearer {api_key_value}"))?;

            auth_value.set_sensitive(true);

            headers.insert(header::AUTHORIZATION, auth_value);
        }

        builder = builder.default_headers(headers);

        Ok(Self {
            client: builder.build()?,
            slots_endpoint_url: Url::parse(&format!("http://{addr}/slots"))?.to_string(),
            models_endpoint_url: Url::parse(&format!("http://{addr}/v1/models"))?.to_string(),
        })
    }

    pub async fn get_available_slots(&self) -> SlotsResponse {
        let url = self.slots_endpoint_url.to_owned();

        let response = match self.client.get(url.clone()).send().await {
            Ok(resp) => resp,
            Err(err) => {
                return SlotsResponse {
                    is_authorized: Some(false),
                    error: Some(format!("Request to {url} Failed. Is it running? {err}")),
                    is_llamacpp_reachable: Some(false),
                    is_llamacpp_response_decodeable: Some(false),
                    is_llamacpp_request_error: Some(true),
                    is_slot_endpoint_enabled: Some(true),
                    slots: vec![],
                }
            }
        };

        let is_server_error = response.status().is_server_error();
        let is_request_error = response.status().is_success();

        match response.status() {
            reqwest::StatusCode::OK => {
                let (slots, err) = match response.json::<Vec<Slot>>().await {
                    Ok(slots) => (Some(slots), None),
                    Err(err) => (None, Some(err)),
                };
                SlotsResponse {
                    is_authorized: Some(true),
                    error: None,
                    is_llamacpp_reachable: Some(is_server_error),
                    is_llamacpp_response_decodeable: Some(err.is_some()),
                    is_llamacpp_request_error: Some(is_request_error),
                    is_slot_endpoint_enabled: Some(true),
                    slots: slots.unwrap_or_default(),
                }
            }
            reqwest::StatusCode::UNAUTHORIZED => SlotsResponse {
                is_authorized: Some(false),
                error: Some("Unauthorized request".into()),
                is_llamacpp_reachable: Some(is_server_error),
                is_llamacpp_response_decodeable: Some(true),
                is_llamacpp_request_error: Some(is_request_error),
                is_slot_endpoint_enabled: None,
                slots: vec![],
            },
            reqwest::StatusCode::NOT_IMPLEMENTED => SlotsResponse {
                is_authorized: None,
                error: Some("Not implemented request".into()),
                is_llamacpp_reachable: Some(is_server_error),
                is_llamacpp_response_decodeable: Some(true),
                is_llamacpp_request_error: Some(is_request_error),
                is_slot_endpoint_enabled: Some(false),
                slots: vec![],
            },
            _ => SlotsResponse {
                is_authorized: None,
                error: Some("Unexpected response status".into()),
                is_llamacpp_reachable: Some(is_server_error),
                is_llamacpp_response_decodeable: Some(true),
                is_llamacpp_request_error: Some(is_request_error),
                is_slot_endpoint_enabled: Some(false),
                slots: vec![],
            },
        }
    }

    pub async fn get_model(&self) -> Result<Option<String>> {
        let url = self.models_endpoint_url.to_owned();

        let response = match self.client.get(url.clone()).send().await {
            Ok(resp) => resp,
            Err(err) => {
                return Err(format!(
                    "Request to '{}' failed: '{}'; connect issue: {}; decode issue: {}; request issue: {}; status issue: {}; status: {:?}",
                    url,
                    err,
                    err.is_connect(),
                    err.is_decode(),
                    err.is_request(),
                    err.is_status(),
                    err.status()
                ).into());
            }
        };

        match response.status() {
            reqwest::StatusCode::OK => {
                let models_response: ModelsResponse = response.json().await?;
                if let Some(models) = models_response.models {
                    if models.is_empty() {
                        Ok(None)
                    } else {
                        Ok(models.first().and_then(|m| Some(m.model.clone())))
                    }
                } else {
                    Ok(None)
                }
            },
            _ => Err("Unexpected response status".into()),
        }
    }
}
