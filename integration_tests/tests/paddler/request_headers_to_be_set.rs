use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;

use crate::cleanable::Cleanable;

#[derive(Debug, Default)]
pub struct RequestHeadersToBeSet {
    pub headers: DashMap<String, Vec<(String, String)>>,
}

impl RequestHeadersToBeSet {
    pub fn insert_header(&self, name: String, header: (String, String)) {
        self.headers.entry(name).or_default().push(header);
    }

    pub fn take_headers_for_request(&self, name: &str) -> Vec<(String, String)> {
        match self.headers.remove(name) {
            Some((_, headers)) => headers,
            None => vec![],
        }
    }
}

#[async_trait]
impl Cleanable for RequestHeadersToBeSet {
    async fn cleanup(&mut self) -> Result<()> {
        self.headers.clear();

        Ok(())
    }
}
