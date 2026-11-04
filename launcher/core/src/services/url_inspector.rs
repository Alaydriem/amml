use crate::{HttpFetch, LauncherError, ModArchive, ModCatalogue, ResolvedMod, UrlInspection};
use amml_gdk::DownloadProgress;

// Downloads a zip from a URL the player typed and finds out what it is.
pub struct UrlInspector;

impl UrlInspector {
    pub fn inspect<H: HttpFetch>(
        catalogue: &mut ModCatalogue<H>,
        url: &str,
        progress: &mut impl FnMut(DownloadProgress),
    ) -> Result<UrlInspection, LauncherError> {
        let file = catalogue.fetch_unchecked(url, progress)?;
        let archive = match ModArchive::open(&file.path) {
            Ok(archive) => archive,
            Err(error) => {
                catalogue.discard(&file);
                return Err(error);
            }
        };
        let mut matched = None;
        let mut mismatch = None;
        for client in catalogue.registries_mut().clients_mut() {
            let Ok(cached) = client.document(&archive.id) else {
                continue;
            };
            if let Some(release) = cached.value.versions.iter().find(|release| release.sha256 == file.sha256).cloned() {
                let registry = client.registry().clone();
                let core = cached.value.core && registry.is_official();
                matched = Some(ResolvedMod { registry, document: cached.value, release, stale: cached.stale, core });
                break;
            }
            // a registry that lists this very URL for other bytes
            if mismatch.is_none()
                && let Some(release) = cached.value.versions.iter().find(|release| release.asset == url)
            {
                mismatch = Some(release.sha256.clone());
            }
        }
        if matched.is_none()
            && let Some(expected) = mismatch
        {
            let found = file.sha256.clone();
            catalogue.discard(&file);
            return Err(LauncherError::ChecksumMismatch { file: url.to_string(), expected, found });
        }
        let api_version = archive.api_version();
        Ok(UrlInspection {
            url: url.to_string(),
            file,
            id: archive.id.clone(),
            listing: archive.manifest.listing.clone(),
            api_version,
            matched,
        })
    }
}
