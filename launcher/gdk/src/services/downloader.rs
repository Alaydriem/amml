use crate::GdkError;
use crate::structs::{CancelFlag, DownloadProgress, RemoteFile};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;

pub struct Downloader {
    agent: ureq::Agent,
    dir: PathBuf,
    // tries per mirror; each try after the first resumes from the partial file
    attempts: u32,
    // set by the app; the stream stops before its next read
    cancel: CancelFlag,
}

impl Downloader {
    const CHUNK: usize = 64 * 1024;
    pub const DEFAULT_ATTEMPTS: u32 = 3;

    pub fn new(agent: ureq::Agent, dir: PathBuf, attempts: u32, cancel: CancelFlag) -> Downloader {
        Downloader { agent, dir, attempts, cancel }
    }

    // the size of the file from a HEAD request, trying each mirror in order
    pub fn remote_size(&self, urls: &[String]) -> Result<u64, GdkError> {
        let mut last = String::from("no download URL");
        for url in urls {
            match self.head(url) {
                Ok(remote) => return Ok(remote.length),
                Err(error) => last = error.to_string(),
            }
        }
        Err(GdkError::Download(last))
    }

    pub fn fetch(
        &self,
        urls: &[String],
        file_name: &str,
        progress: &mut impl FnMut(DownloadProgress),
    ) -> Result<PathBuf, GdkError> {
        self.fetch_limited(urls, file_name, None, progress)
    }

    // Like `fetch`, but a file whose own HEAD answer announces more than `max_len` bytes is refused before any GET.
    // The transfer is then stopped at the announced length, which is at most `max_len`.
    pub fn fetch_limited(
        &self,
        urls: &[String],
        file_name: &str,
        max_len: Option<u64>,
        progress: &mut impl FnMut(DownloadProgress),
    ) -> Result<PathBuf, GdkError> {
        std::fs::create_dir_all(&self.dir)?;
        let mut last = String::from("no download URL");
        for url in urls {
            match self.fetch_from(url, file_name, max_len, progress) {
                Ok(path) => return Ok(path),
                Err(GdkError::Cancelled) => return Err(GdkError::Cancelled),
                Err(error) => {
                    curia::warn!("download source failed", { file: file_name, error: error.to_string() });
                    last = error.to_string();
                }
            }
        }
        Err(GdkError::Download(last))
    }

    fn head(&self, url: &str) -> Result<RemoteFile, GdkError> {
        let response = match self.agent.head(url).header("Accept-Encoding", "identity").call() {
            Ok(response) => response,
            Err(ureq::Error::StatusCode(code)) => return Err(GdkError::Download(format!("HEAD answered {code}"))),
            Err(error) => return Err(GdkError::Download(error.to_string())),
        };
        if !response.status().is_success() {
            return Err(GdkError::Download(format!("HEAD answered {}", response.status().as_u16())));
        }
        let header =
            |name: &str| response.headers().get(name).and_then(|value| value.to_str().ok()).map(str::to_string);
        let length = header("content-length")
            .and_then(|value| value.parse::<u64>().ok())
            .ok_or_else(|| GdkError::Download("the server sent no Content-Length".to_string()))?;
        Ok(RemoteFile {
            length,
            etag: header("etag").unwrap_or_default(),
            last_modified: header("last-modified").unwrap_or_default(),
        })
    }

    fn fetch_from(
        &self,
        url: &str,
        file_name: &str,
        max_len: Option<u64>,
        progress: &mut impl FnMut(DownloadProgress),
    ) -> Result<PathBuf, GdkError> {
        if self.cancel.is_cancelled() {
            return Err(GdkError::Cancelled);
        }
        let dest = self.dir.join(file_name);
        let part = self.dir.join(format!("{file_name}.part"));
        let meta = self.dir.join(format!("{file_name}.part.meta"));
        let remote = self.head(url)?;
        if let Some(max) = max_len
            && remote.length > max
        {
            return Err(GdkError::Download(format!(
                "the server announces {} bytes, more than the limit of {max}",
                remote.length
            )));
        }

        if std::fs::metadata(&dest).is_ok_and(|found| found.len() == remote.length) {
            return Ok(dest);
        }
        self.prepare_part(&part, &meta, &remote)?;

        let mut failure = GdkError::Download("no attempt was made".to_string());
        for _ in 0..self.attempts.max(1) {
            if self.cancel.is_cancelled() {
                return Err(GdkError::Cancelled);
            }
            match self.stream(url, &part, &remote, progress) {
                Ok(()) => {
                    let _ = std::fs::remove_file(&meta);
                    std::fs::rename(&part, &dest)?;
                    return Ok(dest);
                }
                Err(GdkError::Cancelled) => return Err(GdkError::Cancelled),
                Err(error) => failure = error,
            }
        }
        Err(failure)
    }

    // keeps the partial file only when it can continue the remote file
    fn prepare_part(&self, part: &PathBuf, meta: &PathBuf, remote: &RemoteFile) -> Result<(), GdkError> {
        let kept = std::fs::read_to_string(meta).ok().and_then(|text| RemoteFile::from_meta(&text));
        let size = std::fs::metadata(part).map(|found| found.len()).ok();
        let usable = kept.as_ref() == Some(remote) && size.is_some_and(|size| size <= remote.length);
        if !usable {
            let _ = std::fs::remove_file(part);
            std::fs::write(meta, remote.to_meta())?;
        }
        Ok(())
    }

    fn stream(
        &self,
        url: &str,
        part: &PathBuf,
        remote: &RemoteFile,
        progress: &mut impl FnMut(DownloadProgress),
    ) -> Result<(), GdkError> {
        let offset = std::fs::metadata(part).map(|found| found.len()).unwrap_or(0);
        if offset == remote.length {
            return Ok(());
        }
        let mut request = self.agent.get(url).header("Accept-Encoding", "identity");
        if offset > 0 {
            request = request.header("Range", format!("bytes={offset}-"));
        }
        let response = match request.call() {
            Ok(response) => response,
            Err(ureq::Error::StatusCode(code)) => return Err(Self::bad_status(code, part)),
            Err(error) => return Err(GdkError::Download(error.to_string())),
        };
        let status = response.status().as_u16();
        let append = match status {
            206 => {
                // the body must continue exactly where the partial file ends
                let start = response
                    .headers()
                    .get("content-range")
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.strip_prefix("bytes "))
                    .and_then(|value| value.split(['-', '/']).next())
                    .and_then(|value| value.trim().parse::<u64>().ok());
                if start != Some(offset) {
                    let _ = std::fs::remove_file(part);
                    return Err(GdkError::Download("the server answered a different range than asked".to_string()));
                }
                true
            }
            200 => false,
            other => return Err(Self::bad_status(other, part)),
        };
        let mut file = if append {
            OpenOptions::new().append(true).open(part)?
        } else {
            OpenOptions::new().write(true).create(true).truncate(true).open(part)?
        };
        let mut done = if append { offset } else { 0 };
        let mut reader = response.into_body().into_reader();
        let mut buffer = vec![0u8; Self::CHUNK];
        loop {
            if self.cancel.is_cancelled() {
                file.flush()?;
                return Err(GdkError::Cancelled);
            }
            let read = reader.read(&mut buffer).map_err(|error| GdkError::Download(error.to_string()))?;
            if read == 0 {
                break;
            }
            file.write_all(&buffer[..read])?;
            done += read as u64;
            if done > remote.length {
                // a server that sends more than it announced does not get to fill the disk
                drop(file);
                let _ = std::fs::remove_file(part);
                let mut meta = part.clone().into_os_string();
                meta.push(".meta");
                let _ = std::fs::remove_file(meta);
                return Err(GdkError::Download(format!(
                    "the server sent more than the announced {} bytes",
                    remote.length
                )));
            }
            progress(DownloadProgress { bytes_done: done, bytes_total: remote.length });
        }
        file.flush()?;
        if done != remote.length {
            return Err(GdkError::Download(format!("the transfer ended at {done} of {} bytes", remote.length)));
        }
        Ok(())
    }

    // a refused range means the partial file is no use
    fn bad_status(code: u16, part: &PathBuf) -> GdkError {
        if code == 416 {
            let _ = std::fs::remove_file(part);
        }
        GdkError::Download(format!("GET answered {code}"))
    }
}
