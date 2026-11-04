use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use super::{RecordedRequest, Route};

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

    pub fn base(&self) -> String {
        self.url("/")
    }

    pub fn replace(&self, path: &str, route: Route) {
        self.state.lock().unwrap().routes.insert(path.to_string(), route);
    }

    pub fn requests(&self) -> Vec<RecordedRequest> {
        self.state.lock().unwrap().requests.clone()
    }

    pub fn count(&self, path: &str) -> usize {
        self.requests().iter().filter(|request| request.path == path).count()
    }

    pub fn clear_requests(&self) {
        self.state.lock().unwrap().requests.clear();
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
        let headers: Vec<(String, String)> = lines
            .filter_map(|line| line.split_once(':'))
            .map(|(name, value)| (name.trim().to_ascii_lowercase(), value.trim().to_string()))
            .collect();
        let header = |name: &str| headers.iter().find(|(key, _)| key == name).map(|(_, value)| value.clone());
        let if_none_match = header("if-none-match");
        let range = header("range");

        let route = {
            let mut guard = state.lock().unwrap();
            guard.requests.push(RecordedRequest {
                method: method.clone(),
                path: path.clone(),
                if_none_match: if_none_match.clone(),
                range: range.clone(),
            });
            if method == "HEAD"
                && let Some(entry) = guard.routes.get_mut(&path)
                && !entry.head_sequence.is_empty()
            {
                entry.head_length = Some(entry.head_sequence.remove(0));
            }
            guard.routes.get(&path).cloned()
        };
        let Some(route) = route else {
            return Self::respond(&mut stream, "404 Not Found", &[], Some(0), &[]);
        };
        if let Some(delay) = route.delay {
            std::thread::sleep(delay);
        }
        if route.fail {
            return stream.shutdown(Shutdown::Both);
        }
        if let Some(code) = route.status {
            return Self::respond(&mut stream, &format!("{code} Status"), &[], Some(0), &[]);
        }
        let mut headers = vec![("Accept-Ranges".to_string(), "bytes".to_string())];
        if let Some(etag) = &route.etag {
            headers.push(("ETag".to_string(), etag.clone()));
            if if_none_match.as_deref() == Some(etag.as_str()) {
                return Self::respond(&mut stream, "304 Not Modified", &headers, None, &[]);
            }
        }
        if method == "HEAD" {
            return Self::respond(
                &mut stream,
                "200 OK",
                &headers,
                Some(route.head_length.unwrap_or(route.body.len())),
                &[],
            );
        }
        if route.chunked {
            headers.push(("Transfer-Encoding".to_string(), "chunked".to_string()));
            return Self::respond_chunked(&mut stream, &headers, &route.body);
        }
        let start = range
            .as_deref()
            .and_then(|value| value.trim_start_matches("bytes=").trim_end_matches('-').parse::<usize>().ok());
        match start {
            Some(start) if start >= route.body.len() => {
                Self::respond(&mut stream, "416 Range Not Satisfiable", &headers, Some(0), &[])
            }
            Some(start) => {
                headers.push((
                    "Content-Range".to_string(),
                    format!("bytes {start}-{}/{}", route.body.len() - 1, route.body.len()),
                ));
                Self::respond(
                    &mut stream,
                    "206 Partial Content",
                    &headers,
                    Some(route.body.len() - start),
                    &route.body[start..],
                )
            }
            None => Self::respond(&mut stream, "200 OK", &headers, Some(route.body.len()), &route.body),
        }
    }

    fn respond(
        stream: &mut TcpStream,
        status: &str,
        headers: &[(String, String)],
        length: Option<usize>,
        body: &[u8],
    ) -> std::io::Result<()> {
        let mut head = format!("HTTP/1.1 {status}\r\nConnection: close\r\n");
        for (name, value) in headers {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
        if let Some(length) = length {
            head.push_str(&format!("Content-Length: {length}\r\n"));
        }
        head.push_str("\r\n");
        stream.write_all(head.as_bytes())?;
        stream.write_all(body)?;
        stream.flush()?;
        stream.shutdown(Shutdown::Both)
    }

    fn respond_chunked(stream: &mut TcpStream, headers: &[(String, String)], body: &[u8]) -> std::io::Result<()> {
        let mut head = String::from("HTTP/1.1 200 OK\r\nConnection: close\r\n");
        for (name, value) in headers {
            head.push_str(&format!("{name}: {value}\r\n"));
        }
        head.push_str("\r\n");
        stream.write_all(head.as_bytes())?;
        for piece in body.chunks(4096) {
            stream.write_all(format!("{:x}\r\n", piece.len()).as_bytes())?;
            stream.write_all(piece)?;
            stream.write_all(b"\r\n")?;
        }
        stream.write_all(b"0\r\n\r\n")?;
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
