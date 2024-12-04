use reqwest::header;
use std::{net::SocketAddr, time::Duration};
use url::Url;

use crate::{
    errors::result::Result,
    llamacpp::{slot::Slot, slots_response::SlotsResponse},
};

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
            let mut auth_value =
                header::HeaderValue::from_str(&format!("Bearer {}", api_key_value))?;

            auth_value.set_sensitive(true);

            headers.insert(header::AUTHORIZATION, auth_value);
        }

        builder = builder.default_headers(headers);

        Ok(Self {
            client: builder.build()?,
            slots_endpoint_url: Url::parse(&format!("http://{}/slots", addr))?.to_string(),
        })
    }

    pub async fn get_available_slots(&self) -> Result<SlotsResponse> {
        let response = self
            .client
            .get(self.slots_endpoint_url.to_owned())
            .send()
            .await?;

        match response.status() {
            reqwest::StatusCode::OK => Ok(SlotsResponse {
                is_authorized: Some(true),
                is_slot_endpoint_enabled: Some(true),
                slots: response.json::<Vec<Slot>>().await?,
            }),
            reqwest::StatusCode::UNAUTHORIZED => Ok(SlotsResponse {
                is_authorized: Some(false),
                is_slot_endpoint_enabled: None,
                slots: vec![],
            }),
            reqwest::StatusCode::NOT_IMPLEMENTED => Ok(SlotsResponse {
                is_authorized: None,
                is_slot_endpoint_enabled: Some(false),
                slots: vec![],
            }),
            _ => Err("Unexpected response status".into()),
        }
    }
}
