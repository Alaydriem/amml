use super::HttpFetch;
use crate::{Fetched, LauncherError};
use std::io::Read;
use std::time::Duration;

pub struct UreqHttpFetch {
    agent: ureq::Agent,
}

impl UreqHttpFetch {
    const TIMEOUT: Duration = Duration::from_secs(30);

    pub fn new(agent: ureq::Agent) -> UreqHttpFetch {
        UreqHttpFetch { agent }
    }

    // 30 second timeouts and the launcher's user agent; a status other than 2xx is read, not raised
    pub fn agent() -> ureq::Agent {
        ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_connect(Some(Self::TIMEOUT))
            .timeout_recv_response(Some(Self::TIMEOUT))
            .timeout_per_read(Some(Self::TIMEOUT))
            .user_agent(format!("amml-launcher/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .into()
    }

    fn transport(url: &str, error: ureq::Error) -> LauncherError {
        match error {
            ureq::Error::StatusCode(code) => {
                LauncherError::Http { url: url.to_string(), text: format!("status {code}") }
            }
            ureq::Error::BadUri(text) => LauncherError::Http { url: url.to_string(), text },
            _ => LauncherError::Offline { url: url.to_string() },
        }
    }
}

impl HttpFetch for UreqHttpFetch {
    fn get(&self, url: &str, etag: Option<&str>, limit: u64) -> Result<Fetched, LauncherError> {
        let mut request = self.agent.get(url);
        if let Some(etag) = etag {
            request = request.header("If-None-Match", etag);
        }
        let response = match request.call() {
            Ok(response) => response,
            Err(ureq::Error::StatusCode(304)) => return Ok(Fetched::NotModified),
            Err(error) => return Err(Self::transport(url, error)),
        };
        let status = response.status().as_u16();
        if status == 304 {
            return Ok(Fetched::NotModified);
        }
        if !(200..300).contains(&status) {
            return Err(LauncherError::Http { url: url.to_string(), text: format!("status {status}") });
        }
        let declared = response
            .headers()
            .get("content-length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok());
        if declared.is_some_and(|length| length > limit) {
            return Err(LauncherError::TooLarge { url: url.to_string(), limit });
        }
        let etag = response.headers().get("etag").and_then(|value| value.to_str().ok()).map(str::to_string);
        let mut bytes = Vec::new();
        response
            .into_body()
            .into_reader()
            .take(limit + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| LauncherError::Offline { url: url.to_string() })?;
        if bytes.len() as u64 > limit {
            return Err(LauncherError::TooLarge { url: url.to_string(), limit });
        }
        Ok(Fetched::Body { bytes, etag })
    }
}
