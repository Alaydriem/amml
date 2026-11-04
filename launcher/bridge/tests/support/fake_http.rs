use amml_launcher_core::{Fetched, HttpFetch, LauncherError};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

// Registry files served from memory, by full URL. Clones share the files and the record of requests.
#[derive(Clone, Default)]
pub struct FakeHttp {
    files: Arc<Mutex<BTreeMap<String, Vec<u8>>>>,
    requests: Arc<Mutex<Vec<String>>>,
    offline: Arc<AtomicBool>,
}

impl FakeHttp {
    pub fn new() -> FakeHttp {
        FakeHttp::default()
    }

    pub fn put(&self, url: String, bytes: Vec<u8>) {
        self.files.lock().unwrap().insert(url, bytes);
    }

    // every request from now on fails as if the host cannot be reached
    pub fn go_offline(&self) {
        self.offline.store(true, Ordering::SeqCst);
    }

    pub fn requested(&self, url: &str) -> usize {
        self.requests.lock().unwrap().iter().filter(|seen| seen.as_str() == url).count()
    }
}

impl HttpFetch for FakeHttp {
    fn get(&self, url: &str, _etag: Option<&str>, _limit: u64) -> Result<Fetched, LauncherError> {
        self.requests.lock().unwrap().push(url.to_string());
        if self.offline.load(Ordering::SeqCst) {
            return Err(LauncherError::Offline { url: url.to_string() });
        }
        match self.files.lock().unwrap().get(url) {
            Some(bytes) => Ok(Fetched::Body { bytes: bytes.clone(), etag: None }),
            None => Err(LauncherError::Http { url: url.to_string(), text: "404".to_string() }),
        }
    }
}
