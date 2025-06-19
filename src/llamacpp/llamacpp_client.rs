use std::net::SocketAddr;
use std::time::Duration;

use pingora::ErrorTrait;
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

    pub async fn get_available_slots(&self) -> Result<SlotsResponse> {
        let url = self.slots_endpoint_url.to_owned();

        let response = match self.client.get(url.clone()).send().await {
            Ok(resp) => resp,
            Err(err) => {
                return Ok(SlotsResponse {
                    is_authorized: None,
                    error: Some(format!("Request to {} Failed. Is it running?", url)),
                    is_slot_endpoint_enabled: None,
                    is_llamacpp_reachable: Some(!err.is_connect()),
                    is_llamacpp_response_decodeable: Some(!err.is_decode()),
                    is_llamacpp_request_error: Some(err.is_request()),
                    slots: vec![],
                })
            }
        };

        match response.status() {
            reqwest::StatusCode::OK => Ok(SlotsResponse {
                is_authorized: Some(true),
                error: None,
                is_slot_endpoint_enabled: Some(true),
                is_llamacpp_reachable: Some(true),
                is_llamacpp_response_decodeable: Some(true),
                is_llamacpp_request_error: Some(response.status().is_server_error()),
                slots: response.json::<Vec<Slot>>().await?,
            }),
            reqwest::StatusCode::UNAUTHORIZED => Ok(SlotsResponse {
                is_authorized: Some(false),
                error: None,
                is_slot_endpoint_enabled: None,
                is_llamacpp_reachable: Some(true),
                is_llamacpp_response_decodeable: Some(true),
                is_llamacpp_request_error: Some(response.status().is_server_error()),
                slots: vec![],
            }),
            reqwest::StatusCode::NOT_IMPLEMENTED => Ok(SlotsResponse {
                is_authorized: None,
                error: None,
                is_llamacpp_reachable: Some(true),
                is_llamacpp_response_decodeable: Some(true),
                is_llamacpp_request_error: Some(response.status().is_server_error()),
                is_slot_endpoint_enabled: Some(false),
                slots: vec![],
            }),
            _ => Err("Unexpected response status".into()),
        }
    }
}
