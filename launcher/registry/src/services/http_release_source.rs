use std::thread;
use std::time::Duration;

use crate::errors::RegistryError;
use crate::services::ReleaseSource;

/// Downloads release assets over HTTP: `<base>/<repo>/releases/download/<tag>/<asset>`.
pub struct HttpReleaseSource {
    agent: ureq::Agent,
    base: String,
    retries: u32,
    wait: Duration,
}

impl HttpReleaseSource {
    const GITHUB: &'static str = "https://github.com";
    const GITHUB_RETRIES: u32 = 5;
    const GITHUB_WAIT: Duration = Duration::from_secs(20);
    // one request, connect to last byte
    const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);
    // the largest release asset the registry reads
    const MAX_BYTES: u64 = 512 * 1024 * 1024;

    pub fn new(agent: ureq::Agent, base: String, retries: u32, wait: Duration) -> Self {
        Self { agent, base, retries, wait }
    }

    pub fn github() -> Self {
        let agent: ureq::Agent =
            ureq::Agent::config_builder().timeout_global(Some(Self::REQUEST_TIMEOUT)).build().into();
        Self::new(agent, Self::GITHUB.to_string(), Self::GITHUB_RETRIES, Self::GITHUB_WAIT)
    }

    pub fn url(&self, repo: &str, tag: &str, asset: &str) -> String {
        format!("{}/{repo}/releases/download/{tag}/{asset}", self.base)
    }

    fn get(&self, url: &str) -> Result<Option<Vec<u8>>, RegistryError> {
        match self.agent.get(url).call() {
            Ok(mut response) => response
                .body_mut()
                .with_config()
                .limit(Self::MAX_BYTES)
                .read_to_vec()
                .map(Some)
                .map_err(|error| RegistryError::Network(format!("{url}: {error}"))),
            Err(ureq::Error::StatusCode(404)) => Ok(None),
            Err(error) => Err(RegistryError::Network(format!("{url}: {error}"))),
        }
    }
}

impl ReleaseSource for HttpReleaseSource {
    fn asset_url(&self, repo: &str, tag: &str, asset: &str) -> String {
        self.url(repo, tag, asset)
    }

    // a 404 is retried: the asset may not be visible yet right after the release is created
    fn fetch(&self, repo: &str, tag: &str, asset: &str) -> Result<Vec<u8>, RegistryError> {
        let url = self.url(repo, tag, asset);
        for attempt in 0..=self.retries {
            if let Some(bytes) = self.get(&url)? {
                return Ok(bytes);
            }
            if attempt < self.retries {
                thread::sleep(self.wait);
            }
        }
        Err(RegistryError::Missing(asset.to_string()))
    }
}
