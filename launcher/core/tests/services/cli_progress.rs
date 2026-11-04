use amml_gdk::{DownloadProgress, InstallStep};
use amml_launcher_core::{CliProgress, CoreProgress};

fn download(done: u64, total: u64) -> DownloadProgress {
    DownloadProgress { bytes_done: done, bytes_total: total }
}

#[test]
fn a_download_prints_one_line_per_ten_percent_and_each_file_starts_over() {
    let mut printer = CliProgress::new();
    let mut lines = Vec::new();
    for done in (0..=1000).step_by(10) {
        let event = CoreProgress::Downloading { file: "a.zip".to_string(), progress: download(done, 1000) };
        lines.extend(printer.line(&event));
    }
    assert_eq!(lines.len(), 11);
    assert_eq!(lines[0], "Downloading a.zip 0%");
    assert_eq!(lines[10], "Downloading a.zip 100%");

    let other = CoreProgress::Downloading { file: "b.zip".to_string(), progress: download(5, 1000) };
    assert_eq!(printer.line(&other).as_deref(), Some("Downloading b.zip 0%"));
}

#[test]
fn an_empty_total_does_not_divide_by_zero() {
    let mut printer = CliProgress::new();
    assert_eq!(printer.download("x", download(0, 0)).as_deref(), Some("x 0%"));
}

#[test]
fn steps_print_their_label_and_an_install_download_is_indented() {
    let mut printer = CliProgress::new();
    let step = CoreProgress::InstallingVersion { step: InstallStep::Checking, download: None };
    assert_eq!(printer.line(&step).as_deref(), Some(InstallStep::Checking.label()));
    let with_download =
        CoreProgress::InstallingVersion { step: InstallStep::DownloadingWanted, download: Some(download(500, 1000)) };
    assert_eq!(printer.line(&with_download), Some(format!("  {} 50%", InstallStep::DownloadingWanted.label())));
    assert_eq!(printer.line(&with_download), None);
}
