use super::FileHash;
use crate::{Asset, LauncherError, VerifiedFile};
use amml_gdk::{DownloadProgress, Downloader};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

// Downloads a file and checks it against the SHA-256 and size the registry states before anyone opens it.
pub struct AssetFetcher {
    downloader: Downloader,
    dir: PathBuf,
}

impl AssetFetcher {
    const NAME_LIMIT: usize = 128;
    // the largest file of a URL install, where the registry states no size; equal to the unpack limit
    const UNKNOWN_SIZE_LIMIT: u64 = 512 * 1024 * 1024;

    pub fn new(downloader: Downloader, dir: PathBuf) -> AssetFetcher {
        AssetFetcher { downloader, dir }
    }

    pub fn fetch(
        &self,
        asset: &Asset,
        progress: &mut impl FnMut(DownloadProgress),
    ) -> Result<VerifiedFile, LauncherError> {
        let name = Self::file_name(&asset.url);
        let remote = self.downloader.remote_size(std::slice::from_ref(&asset.url))?;
        if asset.size.is_none() && remote > Self::UNKNOWN_SIZE_LIMIT {
            return Err(LauncherError::TooLarge { url: asset.url.clone(), limit: Self::UNKNOWN_SIZE_LIMIT });
        }
        if let Some(expected) = asset.size
            && remote != expected
        {
            return Err(LauncherError::ChecksumMismatch {
                file: name,
                expected: format!("{expected} bytes"),
                found: format!("{remote} bytes"),
            });
        }
        // the downloader asks again, so the limit is passed on and holds whatever the second answer says
        let limit = asset.size.unwrap_or(Self::UNKNOWN_SIZE_LIMIT);
        let path = self.downloader.fetch_limited(std::slice::from_ref(&asset.url), &name, Some(limit), progress)?;
        match self.check(asset, &name, &path) {
            Ok(sha256) => Ok(VerifiedFile { path, sha256 }),
            Err(error) => {
                let _ = std::fs::remove_file(&path);
                Err(error)
            }
        }
    }

    // deletes a download; only a file inside the downloads folder is touched
    pub fn discard(&self, file: &VerifiedFile) {
        if file.path.starts_with(&self.dir) {
            let _ = std::fs::remove_file(&file.path);
        }
    }

    fn check(&self, asset: &Asset, name: &str, path: &std::path::Path) -> Result<String, LauncherError> {
        if let Some(expected) = asset.size {
            let found = std::fs::metadata(path)?.len();
            if found != expected {
                return Err(LauncherError::ChecksumMismatch {
                    file: name.to_string(),
                    expected: format!("{expected} bytes"),
                    found: format!("{found} bytes"),
                });
            }
        }
        let found = FileHash::sha256(path)?;
        if let Some(expected) = &asset.sha256
            && !expected.eq_ignore_ascii_case(&found)
        {
            return Err(LauncherError::ChecksumMismatch { file: name.to_string(), expected: expected.clone(), found });
        }
        Ok(found)
    }

    // the last segment of the URL path when it is a plain `.zip` name, else a name made from the URL's hash
    fn file_name(url: &str) -> String {
        let path = url.split(['?', '#']).next().unwrap_or_default();
        let last = path.rsplit('/').next().unwrap_or_default();
        let plain = last.len() <= Self::NAME_LIMIT
            && last.len() > ".zip".len()
            && last.to_ascii_lowercase().ends_with(".zip")
            && last.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
            && !last.starts_with('.');
        if plain {
            return last.to_string();
        }
        let digest: String = Sha256::digest(url.as_bytes()).iter().take(8).map(|byte| format!("{byte:02x}")).collect();
        format!("url-{digest}.zip")
    }
}
