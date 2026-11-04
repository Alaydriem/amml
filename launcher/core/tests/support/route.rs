use std::time::Duration;

// One served path. The fields are plain switches for the failure a test needs.
#[derive(Clone)]
pub struct Route {
    pub body: Vec<u8>,
    // a request whose If-None-Match equals this gets 304
    pub etag: Option<String>,
    // answers every request with this status and no body
    pub status: Option<u16>,
    // the body is sent with chunked transfer encoding and no Content-Length
    pub chunked: bool,
    // the connection is closed without an answer
    pub fail: bool,
    pub delay: Option<Duration>,
    // what a HEAD request says the length is, when that differs from the body
    pub head_length: Option<usize>,
    // lengths for the next HEAD requests, one each, before `head_length` applies
    pub head_sequence: Vec<usize>,
}

impl Route {
    pub fn new(body: Vec<u8>) -> Route {
        Route {
            body,
            etag: None,
            status: None,
            chunked: false,
            fail: false,
            delay: None,
            head_length: None,
            head_sequence: Vec::new(),
        }
    }

    pub fn with_etag(body: Vec<u8>, etag: &str) -> Route {
        let mut route = Route::new(body);
        route.etag = Some(etag.to_string());
        route
    }
}
