use crate::{Fetched, LauncherError};

// Small registry files over HTTP. A body larger than `limit` is refused.
pub trait HttpFetch {
    fn get(&self, url: &str, etag: Option<&str>, limit: u64) -> Result<Fetched, LauncherError>;
}
