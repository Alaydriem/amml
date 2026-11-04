use crate::support::{Bodies, FakeServer, Route};
use amml_gdk::{CancelFlag, Downloader, GdkError};

const PATH: &str = "/pkg/a.msixvc";

fn downloader(dir: &tempfile::TempDir) -> Downloader {
    Downloader::new(FakeServer::agent(), dir.path().to_path_buf(), Downloader::DEFAULT_ATTEMPTS, CancelFlag::new())
}

fn gets(server: &FakeServer) -> Vec<Option<String>> {
    server.requests().into_iter().filter(|r| r.method == "GET").map(|r| r.range).collect()
}

#[test]
fn resumes_partial_download() {
    let body = Bodies::make(1024 * 1024, 1);
    let mut route = Route::new(body.clone());
    route.etag = Some("\"e1\"".to_string());
    route.drop_after = Some(400 * 1024);
    route.drop_times = 1;
    let server = FakeServer::start(vec![(PATH, route)]);
    let dir = tempfile::tempdir().unwrap();

    let path = downloader(&dir).fetch(&[server.url(PATH)], "a.msixvc", &mut |_| {}).unwrap();

    assert_eq!(std::fs::read(path).unwrap(), body);
    assert_eq!(gets(&server).last().unwrap().as_deref(), Some("bytes=409600-"));
}

#[test]
fn restarts_when_remote_changed() {
    let mut first = Route::new(Bodies::make(2000, 1));
    first.etag = Some("\"old\"".to_string());
    first.drop_after = Some(1500);
    first.drop_times = 99;
    let server = FakeServer::start(vec![(PATH, first)]);
    let dir = tempfile::tempdir().unwrap();
    let loader = Downloader::new(FakeServer::agent(), dir.path().to_path_buf(), 1, CancelFlag::new());
    assert!(loader.fetch(&[server.url(PATH)], "a.msixvc", &mut |_| {}).is_err());
    assert!(dir.path().join("a.msixvc.part").exists());

    let new_body = Bodies::make(2000, 9);
    let mut second = Route::new(new_body.clone());
    second.etag = Some("\"new\"".to_string());
    server.replace(PATH, second);
    let before = server.requests().len();
    let path = loader.fetch(&[server.url(PATH)], "a.msixvc", &mut |_| {}).unwrap();

    assert_eq!(std::fs::read(path).unwrap(), new_body);
    let later: Vec<_> = server.requests().into_iter().skip(before).collect();
    assert!(later.iter().all(|r| r.range.is_none()));
}

#[test]
fn server_ignoring_range_restarts_cleanly() {
    let body = Bodies::make(100_000, 3);
    let mut route = Route::new(body.clone());
    route.etag = Some("\"e\"".to_string());
    route.drop_after = Some(30_000);
    route.drop_times = 1;
    route.ignore_range = true;
    let server = FakeServer::start(vec![(PATH, route)]);
    let dir = tempfile::tempdir().unwrap();

    let path = downloader(&dir).fetch(&[server.url(PATH)], "a.msixvc", &mut |_| {}).unwrap();

    assert_eq!(std::fs::read(path).unwrap(), body);
    assert!(gets(&server).iter().any(|range| range.is_some()));
}

#[test]
fn oversized_part_is_discarded() {
    let mut first = Route::new(Bodies::make(2000, 1));
    first.etag = Some("\"same\"".to_string());
    first.drop_after = Some(1500);
    first.drop_times = 99;
    let server = FakeServer::start(vec![(PATH, first)]);
    let dir = tempfile::tempdir().unwrap();
    let loader = Downloader::new(FakeServer::agent(), dir.path().to_path_buf(), 1, CancelFlag::new());
    assert!(loader.fetch(&[server.url(PATH)], "a.msixvc", &mut |_| {}).is_err());

    let shorter = Bodies::make(1000, 5);
    let mut second = Route::new(shorter.clone());
    second.etag = Some("\"same\"".to_string());
    server.replace(PATH, second);
    let path = loader.fetch(&[server.url(PATH)], "a.msixvc", &mut |_| {}).unwrap();

    assert_eq!(std::fs::read(path).unwrap(), shorter);
}

#[test]
fn falls_back_to_second_mirror() {
    let body = Bodies::make(5000, 2);
    let mut broken = Route::new(Vec::new());
    broken.status = Some(404);
    let first = FakeServer::start(vec![(PATH, broken)]);
    let second = FakeServer::start(vec![(PATH, Route::new(body.clone()))]);
    let dir = tempfile::tempdir().unwrap();

    let path = downloader(&dir).fetch(&[first.url(PATH), second.url(PATH)], "a.msixvc", &mut |_| {}).unwrap();

    assert_eq!(std::fs::read(path).unwrap(), body);
}

#[test]
fn all_mirrors_failing_is_download_error() {
    let mut broken = Route::new(Vec::new());
    broken.status = Some(500);
    let first = FakeServer::start(vec![(PATH, broken.clone())]);
    let second = FakeServer::start(vec![(PATH, broken)]);
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.msixvc.part"), b"kept").unwrap();

    let result = downloader(&dir).fetch(&[first.url(PATH), second.url(PATH)], "a.msixvc", &mut |_| {});

    assert!(matches!(result, Err(GdkError::Download(_))));
    assert_eq!(std::fs::read(dir.path().join("a.msixvc.part")).unwrap(), b"kept");
}

#[test]
fn remote_size_reads_content_length() {
    let server = FakeServer::start(vec![(PATH, Route::new(vec![0u8; 1_048_576]))]);
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(downloader(&dir).remote_size(&[server.url(PATH)]).unwrap(), 1_048_576);
    let requests = server.requests();
    assert_eq!((requests[0].method.as_str(), requests[0].path.as_str()), ("HEAD", PATH));

    let mut silent = Route::new(vec![0u8; 10]);
    silent.content_length_on_head = false;
    let server = FakeServer::start(vec![(PATH, silent)]);
    assert!(matches!(downloader(&dir).remote_size(&[server.url(PATH)]), Err(GdkError::Download(_))));
}

#[test]
fn reports_progress_up_to_the_total() {
    let server = FakeServer::start(vec![(PATH, Route::new(Bodies::make(200_000, 4)))]);
    let dir = tempfile::tempdir().unwrap();
    let mut last = None;
    downloader(&dir).fetch(&[server.url(PATH)], "a.msixvc", &mut |p| last = Some(p)).unwrap();
    let last = last.unwrap();
    assert_eq!((last.bytes_done, last.bytes_total), (200_000, 200_000));
}

#[test]
fn a_range_answer_that_starts_elsewhere_restarts_from_zero() {
    let body = Bodies::make(100_000, 6);
    let mut route = Route::new(body.clone());
    route.etag = Some("\"e\"".to_string());
    route.drop_after = Some(30_000);
    route.drop_times = 1;
    route.wrong_content_range = true;
    let server = FakeServer::start(vec![(PATH, route)]);
    let dir = tempfile::tempdir().unwrap();

    let path = downloader(&dir).fetch(&[server.url(PATH)], "a.msixvc", &mut |_| {}).unwrap();

    assert_eq!(std::fs::read(path).unwrap(), body);
    let ranges = gets(&server);
    assert_eq!(ranges.last().unwrap(), &None);
}

#[test]
fn a_cancelled_download_stops_keeps_the_part_file_and_returns_cancelled() {
    let body = Bodies::make(1024 * 1024, 4);
    let mut route = Route::new(body);
    route.etag = Some("\"e\"".to_string());
    let server = FakeServer::start(vec![(PATH, route)]);
    let dir = tempfile::tempdir().unwrap();
    let cancel = CancelFlag::new();
    let loader =
        Downloader::new(FakeServer::agent(), dir.path().to_path_buf(), Downloader::DEFAULT_ATTEMPTS, cancel.clone());

    let result = loader.fetch(&[server.url(PATH)], "file.bin", &mut |_| cancel.cancel());

    assert!(matches!(result, Err(GdkError::Cancelled)));
    assert!(dir.path().join("file.bin.part").is_file());
    assert!(dir.path().join("file.bin.part.meta").is_file());
    assert_eq!(gets(&server).len(), 1);
}

#[test]
fn a_flag_set_before_the_fetch_sends_no_request() {
    let server = FakeServer::start(vec![(PATH, Route::new(Bodies::make(1000, 2)))]);
    let dir = tempfile::tempdir().unwrap();
    let cancel = CancelFlag::new();
    cancel.cancel();
    let loader = Downloader::new(FakeServer::agent(), dir.path().to_path_buf(), Downloader::DEFAULT_ATTEMPTS, cancel);

    let result = loader.fetch(&[server.url(PATH)], "file.bin", &mut |_| {});

    assert!(matches!(result, Err(GdkError::Cancelled)));
    assert!(server.requests().is_empty());
}

#[test]
fn a_cancelled_download_resumes_from_its_part_file_once_the_flag_is_reset() {
    let body = Bodies::make(1024 * 1024, 6);
    let mut route = Route::new(body.clone());
    route.etag = Some("\"e\"".to_string());
    let server = FakeServer::start(vec![(PATH, route)]);
    let dir = tempfile::tempdir().unwrap();
    let cancel = CancelFlag::new();
    let loader =
        Downloader::new(FakeServer::agent(), dir.path().to_path_buf(), Downloader::DEFAULT_ATTEMPTS, cancel.clone());
    assert!(matches!(
        loader.fetch(&[server.url(PATH)], "file.bin", &mut |_| cancel.cancel()),
        Err(GdkError::Cancelled)
    ));

    cancel.reset();
    let path = loader.fetch(&[server.url(PATH)], "file.bin", &mut |_| {}).unwrap();

    assert_eq!(std::fs::read(path).unwrap(), body);
    assert!(gets(&server).last().unwrap().is_some());
}
