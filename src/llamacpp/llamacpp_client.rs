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
            Ok(resp) => resp,
            Err(err) => {
                let is_reachable = !err.is_connect();
                let is_request_error = err.is_request();
                let is_decodable = !err.is_decode();

                return SlotsResponse {
                    error: Some(format!("Request to {} failed: {}", url, err)),
                    is_authorized: Some(false),
                    is_reachable: Some(is_reachable),
                    is_response_decodeable: Some(is_decodable),
                    is_response_deserializable: None,
                    is_request_error: Some(is_request_error),
                    is_slot_endpoint_enabled: Some(true),
                    slots: vec![],
                };
            }
        };

        let status = response.status();
        let is_reachable = !status.is_server_error();

        let body_result = response.bytes().await;
        let (body, decoding_err) = match &body_result {
            Ok(body) => (Some(body.clone()), None),
            Err(err) => (None, Some(err.to_string())),
        };

        let got_any_bytes = body_result.is_ok();
        let (slots, serializing_err) = match body {
            Some(body_bytes) => match serde_json::from_slice::<Vec<Slot>>(&body_bytes) {
                Ok(slots) => (Some(slots), None),
                Err(err) => (None, Some(err.to_string())),
            },
            None => (None, Some("Empty response body".to_string())),
        };

        let is_successful =
            status.is_success() && decoding_err.is_none() && serializing_err.is_none();
        let is_decodeable = got_any_bytes && decoding_err.is_none();
        let is_deserializable = slots.is_some() && serializing_err.is_none();

        match status {
            reqwest::StatusCode::OK => SlotsResponse {
                is_authorized: Some(true),
                error: None,
                is_reachable: Some(is_reachable),
                is_response_decodeable: Some(is_decodeable),
                is_response_deserializable: Some(is_deserializable),
                is_request_error: Some(!is_successful),
                is_slot_endpoint_enabled: Some(true),
                slots: slots.unwrap_or_default(),
            },
            reqwest::StatusCode::UNAUTHORIZED => SlotsResponse {
                is_authorized: Some(false),
                error: Some("Unauthorized".into()),
                is_reachable: Some(is_reachable),
                is_response_decodeable: Some(is_decodeable),
                is_response_deserializable: Some(is_deserializable),
                is_request_error: Some(true),
                is_slot_endpoint_enabled: None,
                slots: vec![],
            },
            reqwest::StatusCode::NOT_IMPLEMENTED => SlotsResponse {
                is_authorized: Some(false),
                error: Some("Not implemented".into()),
                is_reachable: Some(is_reachable),
                is_response_decodeable: Some(is_decodeable),
                is_response_deserializable: Some(is_deserializable),
                is_request_error: Some(true),
                is_slot_endpoint_enabled: Some(false),
                slots: vec![],
            },
            _ => SlotsResponse {
                is_authorized: Some(false),
                error: Some(format!("Unexpected status: {}", status)),
                is_reachable: Some(is_reachable),
                is_response_decodeable: Some(is_decodeable),
                is_response_deserializable: Some(is_deserializable),
                is_request_error: Some(true),
                is_slot_endpoint_enabled: Some(false),
                slots: vec![],
            },
        }
    }
}
