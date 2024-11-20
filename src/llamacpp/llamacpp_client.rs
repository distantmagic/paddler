use reqwest::header;
use std::{net::SocketAddr, time::Duration};
use url::Url;

use crate::errors::result::Result;
use crate::llamacpp::slot::Slot;

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

    pub async fn get_available_slots(&self) -> Result<Vec<Slot>> {
        let response = self
            .client
            .get(self.slots_endpoint_url.clone())
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Slot>>()
            .await?;

        Ok(response)
    }
}
