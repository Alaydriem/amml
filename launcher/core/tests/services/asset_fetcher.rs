use crate::support::{FakeServer, Route};
use amml_gdk::Downloader;
use amml_launcher_core::{Asset, AssetFetcher, LauncherError, UreqHttpFetch};
use amml_registry::HashText;

fn fetcher(dir: &std::path::Path) -> AssetFetcher {
    let downloader = Downloader::new(UreqHttpFetch::agent(), dir.to_path_buf(), 1);
    AssetFetcher::new(downloader, dir.to_path_buf())
}

fn leftovers(dir: &std::path::Path) -> Vec<String> {
    std::fs::read_dir(dir).unwrap().map(|entry| entry.unwrap().file_name().to_string_lossy().to_string()).collect()
}

#[test]
fn a_matching_hash_gives_the_file() {
    let body = vec![9u8; 5000];
    let server = FakeServer::start(vec![("/files/zoom-1.zip", Route::new(body.clone()))]);
    let dir = tempfile::tempdir().unwrap();
    let fetcher = fetcher(dir.path());
    let asset = Asset {
        url: server.url("/files/zoom-1.zip"),
        sha256: Some(HashText::sha256(&body)),
        size: Some(body.len() as u64),
    };
    let mut seen = 0;
    let file = fetcher.fetch(&asset, &mut |_| seen += 1).unwrap();
    assert_eq!(file.path, dir.path().join("zoom-1.zip"));
    assert_eq!(file.sha256, HashText::sha256(&body));
    assert!(seen > 0);
    assert_eq!(std::fs::read(&file.path).unwrap(), body);

    fetcher.discard(&file);
    assert!(leftovers(dir.path()).is_empty());
}

#[test]
fn a_wrong_hash_deletes_the_download_and_names_both_hashes() {
    let body = vec![9u8; 5000];
    let server = FakeServer::start(vec![("/zoom.zip", Route::new(body.clone()))]);
    let dir = tempfile::tempdir().unwrap();
    let asset = Asset { url: server.url("/zoom.zip"), sha256: Some("ab".repeat(32)), size: Some(5000) };
    match fetcher(dir.path()).fetch(&asset, &mut |_| {}) {
        Err(LauncherError::ChecksumMismatch { file, expected, found }) => {
            assert_eq!(file, "zoom.zip");
            assert_eq!(expected, "ab".repeat(32));
            assert_eq!(found, HashText::sha256(&body));
        }
        other => panic!("{other:?}"),
    }
    assert!(leftovers(dir.path()).is_empty());
}

#[test]
fn a_size_other_than_the_registry_size_is_refused() {
    let body = vec![9u8; 5000];
    let server = FakeServer::start(vec![("/zoom.zip", Route::new(body.clone()))]);
    let dir = tempfile::tempdir().unwrap();
    let asset = Asset { url: server.url("/zoom.zip"), sha256: Some(HashText::sha256(&body)), size: Some(4999) };
    assert!(matches!(fetcher(dir.path()).fetch(&asset, &mut |_| {}), Err(LauncherError::ChecksumMismatch { .. })));
    assert!(leftovers(dir.path()).is_empty());
}

#[test]
fn an_odd_url_gets_a_hashed_file_name() {
    let body = vec![1u8; 100];
    let server = FakeServer::start(vec![("/download?id=5", Route::new(body))]);
    let dir = tempfile::tempdir().unwrap();
    let asset = Asset { url: server.url("/download?id=5"), sha256: None, size: None };
    let file = fetcher(dir.path()).fetch(&asset, &mut |_| {}).unwrap();
    let name = file.path.file_name().unwrap().to_string_lossy().to_string();
    assert!(name.starts_with("url-") && name.ends_with(".zip"), "{name}");
    assert_eq!(name.len(), "url-".len() + 16 + ".zip".len());
}

#[test]
fn discard_leaves_a_file_outside_the_downloads_folder_alone() {
    let dir = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let path = elsewhere.path().join("keep.zip");
    std::fs::write(&path, b"x").unwrap();
    fetcher(dir.path()).discard(&amml_launcher_core::VerifiedFile { path: path.clone(), sha256: String::new() });
    assert!(path.exists());
}

#[test]
fn a_download_that_goes_past_its_announced_length_is_stopped_and_leaves_no_file() {
    let mut route = Route::new(vec![3u8; 5000]);
    route.head_length = Some(1000);
    let server = FakeServer::start(vec![("/big.zip", route)]);
    let dir = tempfile::tempdir().unwrap();
    let asset = Asset { url: server.url("/big.zip"), sha256: None, size: Some(1000) };
    assert!(matches!(fetcher(dir.path()).fetch(&asset, &mut |_| {}), Err(LauncherError::Gdk(_))));
    assert!(leftovers(dir.path()).is_empty(), "{:?}", leftovers(dir.path()));
}

#[test]
fn a_url_install_with_no_stated_size_has_a_limit() {
    let mut route = Route::new(Vec::new());
    route.head_length = Some(600 * 1024 * 1024);
    let server = FakeServer::start(vec![("/huge.zip", route)]);
    let dir = tempfile::tempdir().unwrap();
    let asset = Asset { url: server.url("/huge.zip"), sha256: None, size: None };
    match fetcher(dir.path()).fetch(&asset, &mut |_| {}) {
        Err(LauncherError::TooLarge { limit, .. }) => assert_eq!(limit, 512 * 1024 * 1024),
        other => panic!("{other:?}"),
    }
    assert_eq!(server.count("/huge.zip"), 1);
    assert!(leftovers(dir.path()).is_empty());
}

#[test]
fn a_second_head_that_announces_more_than_the_limit_is_refused_before_any_get() {
    for size in [Some(1000), None] {
        let mut route = Route::new(vec![3u8; 5000]);
        // the first HEAD (the fetcher's own check) is small, the second (the downloader's) is far over the limit
        route.head_sequence = vec![1000, 900 * 1024 * 1024];
        let server = FakeServer::start(vec![("/swap.zip", route)]);
        let dir = tempfile::tempdir().unwrap();
        let asset = Asset { url: server.url("/swap.zip"), sha256: None, size };
        let result = fetcher(dir.path()).fetch(&asset, &mut |_| {});
        assert!(matches!(result, Err(LauncherError::Gdk(_))), "{size:?}: {result:?}");
        let gets = server.requests().iter().filter(|request| request.method == "GET").count();
        assert_eq!(gets, 0, "{size:?}");
        assert!(leftovers(dir.path()).is_empty(), "{size:?}");
    }
}
