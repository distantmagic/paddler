use reqwest::header;
use std::time::Duration;

use crate::errors::result::Result;
use crate::llamacpp::slot::Slot;

pub struct LlamacppClient {
    addr: url::Url,
    client: reqwest::Client,
}

impl LlamacppClient {
    pub fn new(addr: url::Url, api_key: Option<String>) -> Result<Self> {
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
            addr,
            client: builder.build()?,
        })
    }

    pub async fn get_available_slots(&self) -> Result<Vec<Slot>> {
        let response = self
            .client
            .get(self.addr.join("/slots")?.as_str())
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Slot>>()
            .await?;

        Ok(response)
    }
}
