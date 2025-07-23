use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;

use crate::cleanable::Cleanable;
use crate::request_headers_to_be_set::RequestHeadersToBeSet;

#[derive(Debug, Default)]
pub struct RequestBuilder {
    pub headers_to_be_set: RequestHeadersToBeSet,
}

impl RequestBuilder {
    pub fn get(&self, name: &str, path: String) -> reqwest::RequestBuilder {
        let mut request_builder = Client::new().get(path);

        let headers = self.headers_to_be_set.take_headers_for_request(name);

        for (key, value) in headers {
            request_builder = request_builder.header(key, value);
        }

        request_builder
    }
}

#[async_trait]
impl Cleanable for RequestBuilder {
    async fn cleanup(&mut self) -> Result<()> {
        self.headers_to_be_set.cleanup().await
    }
}
