use reqwest::header;
use std::{net::SocketAddr, time::Duration};
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

        builder = match api_key {
            Some(api_key) => {
                let mut headers = header::HeaderMap::new();
                let mut auth_value = header::HeaderValue::from_str(&format!("Bearer {}", api_key))?;

                auth_value.set_sensitive(true);

                headers.insert(header::AUTHORIZATION, auth_value);

                builder.default_headers(headers)
            }
            None => builder,
        };

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
                is_authorized: true,
                slots: response.json::<Vec<Slot>>().await?,
            }),
            reqwest::StatusCode::UNAUTHORIZED => Ok(SlotsResponse {
                is_authorized: false,
                slots: vec![],
            }),
            _ => Err("Unexpected response status".into()),
        }
    }
}
