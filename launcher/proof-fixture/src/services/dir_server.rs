use std::fs;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

/// Serves the files under a folder over HTTP on 127.0.0.1 and records every request path.
pub struct DirServer {
    port: u16,
    requests: Arc<Mutex<Vec<String>>>,
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl DirServer {
    pub fn start(root: &Path) -> DirServer {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let handle = {
            let (root, requests, stop) = (root.to_path_buf(), requests.clone(), stop.clone());
            std::thread::spawn(move || {
                for stream in listener.incoming() {
                    if stop.load(Ordering::SeqCst) {
                        break;
                    }
                    if let Ok(stream) = stream {
                        let _ = Self::serve(stream, &root, &requests);
                    }
                }
            })
        };
        DirServer { port, requests, stop, handle: Some(handle) }
    }

    pub fn base(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// The request paths, in the order they arrived.
    pub fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }

    fn serve(mut stream: TcpStream, root: &Path, requests: &Mutex<Vec<String>>) -> std::io::Result<()> {
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
        let mut first = text.lines().next().unwrap_or_default().split(' ');
        let method = first.next().unwrap_or_default();
        let path = first.next().unwrap_or_default().to_string();
        requests.lock().unwrap().push(path.clone());
        let body = Self::resolve(root, &path).and_then(|file| fs::read(file).ok());
        let (status, body) = match body {
            Some(body) => ("200 OK", body),
            None => ("404 Not Found", Vec::new()),
        };
        let head = format!("HTTP/1.1 {status}\r\nConnection: close\r\nContent-Length: {}\r\n\r\n", body.len());
        stream.write_all(head.as_bytes())?;
        if method != "HEAD" {
            stream.write_all(&body)?;
        }
        stream.flush()?;
        stream.shutdown(Shutdown::Both)
    }

    // a request path inside the root, or None for a path that climbs out of it
    fn resolve(root: &Path, request: &str) -> Option<PathBuf> {
        let path = request.split(['?', '#']).next().unwrap_or_default();
        let mut file = root.to_path_buf();
        for segment in path.split('/').filter(|segment| !segment.is_empty()) {
            if segment == ".." || segment == "." || segment.contains(['\\', ':']) {
                return None;
            }
            file.push(segment);
        }
        file.is_file().then_some(file)
    }
}

impl Drop for DirServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(("127.0.0.1", self.port));
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
