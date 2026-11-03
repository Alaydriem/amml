use crate::support::{FakeServer, Route};
use amml_launcher_core::{Fetched, HttpFetch, LauncherError, UreqHttpFetch};

fn fetcher() -> UreqHttpFetch {
    UreqHttpFetch::new(UreqHttpFetch::agent())
}

#[test]
fn an_etag_match_is_not_modified() {
    let server = FakeServer::start(vec![("/a.json", Route::with_etag(b"{}".to_vec(), "\"v1\""))]);
    let url = server.url("/a.json");
    let first = fetcher().get(&url, None, 1000).unwrap();
    let Fetched::Body { bytes, etag } = first else { panic!("expected a body") };
    assert_eq!(bytes, b"{}");
    assert_eq!(etag.as_deref(), Some("\"v1\""));

    let second = fetcher().get(&url, etag.as_deref(), 1000).unwrap();
    assert_eq!(second, Fetched::NotModified);
    let seen = server.requests();
    assert_eq!(seen[1].if_none_match.as_deref(), Some("\"v1\""));
}

#[test]
fn a_body_over_the_limit_is_refused_without_a_content_length() {
    let mut route = Route::new(vec![7u8; 70 * 1024]);
    route.chunked = true;
    let server = FakeServer::start(vec![("/big", route)]);
    match fetcher().get(&server.url("/big"), None, 65_536) {
        Err(LauncherError::TooLarge { limit, .. }) => assert_eq!(limit, 65_536),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_body_over_the_limit_is_refused_by_its_content_length() {
    let server = FakeServer::start(vec![("/big", Route::new(vec![7u8; 2000]))]);
    assert!(matches!(fetcher().get(&server.url("/big"), None, 1000), Err(LauncherError::TooLarge { .. })));
}

#[test]
fn a_closed_port_is_offline() {
    let server = FakeServer::start(vec![]);
    let url = server.url("/x");
    drop(server);
    assert!(matches!(fetcher().get(&url, None, 1000), Err(LauncherError::Offline { .. })));
}

#[test]
fn a_404_is_an_http_error_naming_the_url() {
    let server = FakeServer::start(vec![]);
    let url = server.url("/missing");
    match fetcher().get(&url, None, 1000) {
        Err(LauncherError::Http { url: named, text }) => {
            assert_eq!(named, url);
            assert!(text.contains("404"), "{text}");
        }
        other => panic!("{other:?}"),
    }
}
