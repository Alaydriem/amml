#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadProgress {
    pub bytes_done: u64,
    pub bytes_total: u64,
}
