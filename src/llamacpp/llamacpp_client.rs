use std::net::SocketAddr;
use std::time::Duration;

use reqwest::header;
use url::Url;

use crate::errors::result::Result;
use crate::llamacpp::slot::Slot;
use crate::llamacpp::slots_response::SlotsResponse;

pub struct LlamacppClient {
    client: reqwest::Client,
    slots_endpoint_url: String,
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
        })
    }

    pub async fn get_available_slots(&self) -> SlotsResponse {
        let url = self.slots_endpoint_url.clone();
    
        let response = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                return SlotsResponse {
                    error: Some(format!("Request to {} failed: {}", url, e)),
                    is_llamacpp_reachable: Some(!e.is_connect()),
                    is_llamacpp_response_decodeable: Some(!e.is_decode()),
                    is_llamacpp_request_error: Some(e.is_request()),
                    is_slot_endpoint_enabled: None,
                    is_authorized: None,
                    slots: vec![],
                };
            }
        };
    
        let status = response.status();
        let reachable = Some(true);
        let decodeable = Some(true);
        let request_error = Some(status.is_server_error());
    
        match status {
            reqwest::StatusCode::OK => {
                let slots = response.json::<Vec<Slot>>().await.unwrap_or_else(|e| {
                    eprintln!("JSON decode failed: {}", e);
                    vec![]
                });
    
                SlotsResponse {
                    error: None,
                    is_llamacpp_reachable: reachable,
                    is_llamacpp_response_decodeable: decodeable,
                    is_llamacpp_request_error: Some(false),
                    is_slot_endpoint_enabled: Some(true),
                    is_authorized: Some(true),
                    slots,
                }
            }
            reqwest::StatusCode::UNAUTHORIZED => SlotsResponse {
                error: None,
                is_llamacpp_reachable: reachable,
                is_llamacpp_response_decodeable: decodeable,
                is_llamacpp_request_error: Some(false),
                is_slot_endpoint_enabled: None,
                is_authorized: Some(false),
                slots: vec![],
            },
            reqwest::StatusCode::NOT_IMPLEMENTED => SlotsResponse {
                error: None,
                is_llamacpp_reachable: reachable,
                is_llamacpp_response_decodeable: decodeable,
                is_llamacpp_request_error: Some(false),
                is_slot_endpoint_enabled: Some(false),
                is_authorized: None,
                slots: vec![],
            },
            _ => SlotsResponse {
                error: Some(format!("Unexpected status: {}", status)),
                is_llamacpp_reachable: reachable,
                is_llamacpp_response_decodeable: decodeable,
                is_llamacpp_request_error: request_error,
                is_slot_endpoint_enabled: Some(false),
                is_authorized: None,
                slots: vec![],
            },
        }
    }
    
    
}
