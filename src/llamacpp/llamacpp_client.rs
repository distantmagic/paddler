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
        let url = self.slots_endpoint_url.to_owned();

        let response = match self.client.get(url.clone()).send().await {
            Ok(resp) => resp,
            Err(err) => {
                return SlotsResponse {
                    error: Some(format!("Request to {url} Failed. Is it running? {err}")),
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

<<<<<<< HEAD
        let is_llamacpp_reachable = !response.status().is_server_error();
        let is_request_error = response.status().is_server_error();
=======
        let status = response.status();
>>>>>>> b74bf2bc7563e85667314dbc1ae0b39d655e2fbc

        match status {
            reqwest::StatusCode::OK => {
                let (slots, error) = match response.json::<Vec<Slot>>().await {
                    Ok(slots) => (Some(slots), None),
                    Err(err) => (None, Some(err.to_string())),
                };

                SlotsResponse {
                    error: error.clone(),
                    is_authorized: Some(true),
<<<<<<< HEAD
                    error: None,
                    is_llamacpp_reachable: Some(is_llamacpp_reachable),
                    is_llamacpp_response_decodeable: Some(err.is_none()),
                    is_llamacpp_request_error: Some(is_request_error),
=======
                    is_unexpected_response_status: Some(false),
                    is_connect_error: Some(false),
                    is_decode_error: Some(false),
                    is_deserialize_error: Some(error.is_some()),
                    is_request_error: Some(false),
>>>>>>> b74bf2bc7563e85667314dbc1ae0b39d655e2fbc
                    is_slot_endpoint_enabled: Some(true),
                    slots: slots.unwrap_or_default(),
                }
            }
            reqwest::StatusCode::UNAUTHORIZED => SlotsResponse {
                error: Some("Unauthorized".into()),
                is_authorized: Some(false),
<<<<<<< HEAD
                error: Some("Unauthorized request".into()),
                is_llamacpp_reachable: Some(is_llamacpp_reachable),
                is_llamacpp_response_decodeable: Some(true),
                is_llamacpp_request_error: Some(is_request_error),
=======
                is_unexpected_response_status: Some(false),
                is_connect_error: Some(false),
                is_decode_error: Some(false),
                is_deserialize_error: None,
                is_request_error: Some(false),
>>>>>>> b74bf2bc7563e85667314dbc1ae0b39d655e2fbc
                is_slot_endpoint_enabled: None,
                slots: vec![],
            },
            reqwest::StatusCode::NOT_IMPLEMENTED => SlotsResponse {
                error: Some("Not implemented".into()),
                is_authorized: None,
<<<<<<< HEAD
                error: Some("Not implemented request".into()),
                is_llamacpp_reachable: Some(is_llamacpp_reachable),
                is_llamacpp_response_decodeable: Some(true),
                is_llamacpp_request_error: Some(is_request_error),
=======
                is_unexpected_response_status: Some(false),
                is_connect_error: Some(false),
                is_decode_error: Some(false),
                is_deserialize_error: None,
                is_request_error: Some(false),
>>>>>>> b74bf2bc7563e85667314dbc1ae0b39d655e2fbc
                is_slot_endpoint_enabled: Some(false),
                slots: vec![],
            },
            _ => SlotsResponse {
                error: Some("Unexpected response status".into()),
<<<<<<< HEAD
                is_llamacpp_reachable: Some(is_llamacpp_reachable),
                is_llamacpp_response_decodeable: Some(true),
                is_llamacpp_request_error: Some(is_request_error),
                is_slot_endpoint_enabled: Some(false),
=======
                is_authorized: None,
                is_unexpected_response_status: Some(true),
                is_connect_error: Some(false),
                is_decode_error: Some(false),
                is_deserialize_error: None,
                is_request_error: Some(false),
                is_slot_endpoint_enabled: None,
>>>>>>> b74bf2bc7563e85667314dbc1ae0b39d655e2fbc
                slots: vec![],
            },
        }
    }
}
