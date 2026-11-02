use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

// One served path. The fields are plain switches for the failure a test needs.
#[derive(Clone)]
pub struct Route {
    pub body: Vec<u8>,
    pub etag: Option<String>,
    // answers every request with this status and no body
    pub status: Option<u16>,
    pub ignore_range: bool,
    // a ranged answer says it starts at byte 0 whatever it sends
    pub wrong_content_range: bool,
    pub content_length_on_head: bool,
    // a GET sends only this many body bytes, then closes the connection
    pub drop_after: Option<usize>,
    // how many GET requests are cut short
    pub drop_times: usize,
}

impl Route {
    pub fn new(body: Vec<u8>) -> Route {
        Route {
            body,
            etag: None,
            status: None,
            ignore_range: false,
            wrong_content_range: false,
            content_length_on_head: true,
            drop_after: None,
            drop_times: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecordedRequest {
    pub method: String,
    pub path: String,
    pub range: Option<String>,
}

struct State {
    routes: HashMap<String, Route>,
    requests: Vec<RecordedRequest>,
}

pub struct FakeServer {
    port: u16,
    state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl FakeServer {
    pub fn start(routes: Vec<(&str, Route)>) -> FakeServer {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let routes = routes.into_iter().map(|(path, route)| (path.to_string(), route)).collect();
        let state = Arc::new(Mutex::new(State { routes, requests: Vec::new() }));
        let stop = Arc::new(AtomicBool::new(false));
        let handle = {
            let state = state.clone();
            let stop = stop.clone();
            std::thread::spawn(move || {
                for stream in listener.incoming() {
                    if stop.load(Ordering::SeqCst) {
                        break;
                    }
                    if let Ok(stream) = stream {
                        let _ = Self::serve(stream, &state);
                    }
                }
            })
        };
        FakeServer { port, state, stop, handle: Some(handle) }
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{}", self.port, path)
    }

    pub fn replace(&self, path: &str, route: Route) {
        self.state.lock().unwrap().routes.insert(path.to_string(), route);
    }

    pub fn requests(&self) -> Vec<RecordedRequest> {
        self.state.lock().unwrap().requests.clone()
    }

    pub fn agent() -> ureq::Agent {
        ureq::Agent::config_builder().http_status_as_error(false).build().into()
    }

    fn serve(mut stream: TcpStream, state: &Arc<Mutex<State>>) -> std::io::Result<()> {
        stream.set_read_timeout(Some(Duration::from_secs(5)))?;
        let mut raw = Vec::new();
        let mut chunk = [0u8; 1024];
        while !raw.windows(4).any(|window| window == b"\r\n\r\n") {
            let read = stream.read(&mut chunk)?;
            if read == 0 {
                return Ok(());
            }
            raw.extend_from_slice(&chunk[..read]);
        }
        let text = String::from_utf8_lossy(&raw).to_string();
        let mut lines = text.lines();
        let mut first = lines.next().unwrap_or_default().split(' ');
        let method = first.next().unwrap_or_default().to_string();
        let path = first.next().unwrap_or_default().to_string();
        let range = lines
            .filter_map(|line| line.split_once(':'))
            .find(|(name, _)| name.eq_ignore_ascii_case("range"))
            .map(|(_, value)| value.trim().to_string());

        let route = {
            let mut guard = state.lock().unwrap();
            guard.requests.push(RecordedRequest { method: method.clone(), path: path.clone(), range: range.clone() });
            let route = guard.routes.get(&path).cloned();
            if method == "GET"
                && let Some(entry) = guard.routes.get_mut(&path)
                && entry.drop_times > 0
            {
                entry.drop_times -= 1;
            }
            route
        };
        let Some(route) = route else {
            return Self::respond(&mut stream, "404 Not Found", &[], 0, &[]);
        };
        if let Some(code) = route.status {
            return Self::respond(&mut stream, &format!("{code} Status"), &[], 0, &[]);
        }
        let mut headers = vec![("Accept-Ranges".to_string(), "bytes".to_string())];
        if let Some(etag) = &route.etag {
            headers.push(("ETag".to_string(), etag.clone()));
        }
        if method == "HEAD" {
            let length = if route.content_length_on_head { route.body.len() } else { usize::MAX };
            return Self::respond(&mut stream, "200 OK", &headers, length, &[]);
        }
        let start = match (&range, route.ignore_range) {
            (Some(value), false) => value.trim_start_matches("bytes=").trim_end_matches('-').parse::<usize>().ok(),
            _ => None,
        };
        let (status, body) = match start {
            Some(start) if start >= route.body.len() => {
                return Self::respond(&mut stream, "416 Range Not Satisfiable", &headers, 0, &[]);
            }
            Some(start) => {
                let announced = if route.wrong_content_range { 0 } else { start };
                headers.push((
                    "Content-Range".to_string(),
                    format!("bytes {announced}-{}/{}", route.body.len() - 1, route.body.len()),
                ));
                ("206 Partial Content", &route.body[start..])
            }
            None => ("200 OK", &route.body[..]),
        };
        let drops = route.drop_times > 0;
        let sent = match (drops, route.drop_after) {
            (true, Some(limit)) => &body[..limit.min(body.len())],
            _ => body,
        };
        Self::respond(&mut stream, status, &headers, body.len(), sent)
    }

    fn respond(
        stream: &mut TcpStream,
        status: &str,
        headers: &[(String, String)],
        length: usize,
        body: &[u8],
    ) -> std::io::Result<()> {
        let mut head = format!("HTTP/1.1 {status}\r\nConnection: close\r\n");
        for (name, value) in headers {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
        if length != usize::MAX {
            head.push_str(&format!("Content-Length: {length}\r\n"));
        }
        head.push_str("\r\n");
        stream.write_all(head.as_bytes())?;
        stream.write_all(body)?;
        stream.flush()?;
        stream.shutdown(Shutdown::Both)
    }
}

impl Drop for FakeServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(("127.0.0.1", self.port));
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
