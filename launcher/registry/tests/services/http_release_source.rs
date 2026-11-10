use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use amml_registry::{HttpReleaseSource, RegistryError, ReleaseSource};

/// Answers each request with the next status line and body, then closes; returns the base URL.
fn serve(answers: Vec<(&'static str, &'static str)>) -> (String, thread::JoinHandle<usize>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let mut served = 0;
        for (status, body) in answers {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 2048];
            let _ = stream.read(&mut buffer);
            let reply =
                format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            stream.write_all(reply.as_bytes()).unwrap();
            served += 1;
        }
        served
    });
    (base, handle)
}

fn source(base: String, retries: u32) -> HttpReleaseSource {
    HttpReleaseSource::new(ureq::Agent::new_with_defaults(), base, retries, Duration::from_millis(5))
}

#[test]
fn builds_the_github_download_url() {
    let source = HttpReleaseSource::github();
    assert_eq!(
        source.url("alaydriem/amml-zoom", "26.60.1", "SHA256SUMS.txt"),
        "https://github.com/alaydriem/amml-zoom/releases/download/26.60.1/SHA256SUMS.txt"
    );
}

#[test]
fn a_404_is_retried_until_the_asset_shows_up() {
    let (base, handle) = serve(vec![("404 Not Found", ""), ("404 Not Found", ""), ("200 OK", "payload")]);
    let bytes = source(base, 5).fetch("o/r", "1.0.0", "a.zip").unwrap();
    assert_eq!(bytes, b"payload");
    assert_eq!(handle.join().unwrap(), 3);
}

#[test]
fn a_404_that_stays_is_missing_after_the_bounded_retries() {
    let (base, handle) = serve(vec![("404 Not Found", ""); 3]);
    match source(base, 2).fetch("o/r", "1.0.0", "a.zip") {
        Err(RegistryError::Missing(asset)) => assert_eq!(asset, "a.zip"),
        other => panic!("{other:?}"),
    }
    assert_eq!(handle.join().unwrap(), 3);
}

#[test]
fn another_error_status_is_not_retried() {
    let (base, handle) = serve(vec![("500 Internal Server Error", "")]);
    assert!(matches!(source(base, 5).fetch("o/r", "1.0.0", "a.zip"), Err(RegistryError::Network(_))));
    assert_eq!(handle.join().unwrap(), 1);
}

#[test]
fn a_server_that_never_answers_is_a_network_error_after_the_timeout() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let agent: ureq::Agent =
        ureq::Agent::config_builder().timeout_global(Some(Duration::from_millis(200))).build().into();
    let source = HttpReleaseSource::new(agent, base, 0, Duration::from_millis(1));
    assert!(matches!(source.fetch("o/r", "1.0.0", "a.zip"), Err(RegistryError::Network(_))));
    drop(listener);
}

#[test]
fn github_source_writes_the_same_asset_url_as_before() {
    assert_eq!(
        HttpReleaseSource::github().asset_url("alaydriem/amml-heads", "26.60.6", "heads-26.60.6-windows-x64.zip"),
        "https://github.com/alaydriem/amml-heads/releases/download/26.60.6/heads-26.60.6-windows-x64.zip"
    );
}
