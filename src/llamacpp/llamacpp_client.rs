use std::net::SocketAddr;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Result;
use reqwest::header;
use url::Url;

use crate::llamacpp::slot::Slot;
use crate::llamacpp::slots_response::SlotsResponse;
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
                    error: Some(format!("Request to {url} failed: {err}")),
                    is_authorized: None,
                    is_connect_error: Some(err.is_connect()),
                    is_decode_error: Some(err.is_decode()),
                    is_deserialize_error: None,
                    is_request_error: Some(err.is_request()),
                    is_unexpected_response_status: None,
                    is_slot_endpoint_enabled: Some(true),
                    slots: vec![],
                };
            }
        };

        let status = response.status();

        match status {
            reqwest::StatusCode::OK => {
                let (slots, error) = match response.json::<Vec<Slot>>().await {
                    Ok(slots) => (Some(slots), None),
                    Err(err) => (None, Some(err.to_string())),
                };

                SlotsResponse {
                    error: error.clone(),
                    is_authorized: Some(true),
                    is_unexpected_response_status: Some(false),
                    is_connect_error: Some(false),
                    is_decode_error: Some(false),
                    is_deserialize_error: Some(error.is_some()),
                    is_request_error: Some(false),
                    is_slot_endpoint_enabled: Some(true),
                    slots: slots.unwrap_or_default(),
                }
            }
            reqwest::StatusCode::UNAUTHORIZED => SlotsResponse {
                error: Some("Unauthorized".into()),
                is_authorized: Some(false),
                is_unexpected_response_status: Some(false),
                is_connect_error: Some(false),
                is_decode_error: Some(false),
                is_deserialize_error: None,
                is_request_error: Some(false),
                is_slot_endpoint_enabled: None,
                slots: vec![],
            },
            reqwest::StatusCode::NOT_IMPLEMENTED => SlotsResponse {
                error: Some("Not implemented".into()),
                is_authorized: None,
                is_unexpected_response_status: Some(false),
                is_connect_error: Some(false),
                is_decode_error: Some(false),
                is_deserialize_error: None,
                is_request_error: Some(false),
                is_slot_endpoint_enabled: Some(false),
                slots: vec![],
            },
            _ => SlotsResponse {
                error: Some("Unexpected response status".into()),
                is_authorized: None,
                is_unexpected_response_status: Some(true),
                is_connect_error: Some(false),
                is_decode_error: Some(false),
                is_deserialize_error: None,
                is_request_error: Some(false),
                is_slot_endpoint_enabled: None,
                slots: vec![],
            },
        }
    }

    pub async fn get_model(&self) -> Result<Option<String>> {
        let url = self.models_endpoint_url.to_owned();

        let response = match self.client.get(url.clone()).send().await {
            Ok(resp) => resp,
            Err(err) => {
                return Err(anyhow!(
                    "Request to '{}' failed: '{}'; connect issue: {}; decode issue: {}; request issue: {}; status issue: {}; status: {:?}",
                    url,
                    err,
                    err.is_connect(),
                    err.is_decode(),
                    err.is_request(),
                    err.is_status(),
                    err.status()
                ));
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
            _ => Err(anyhow!("Unexpected response status")),
        }
    }
}
