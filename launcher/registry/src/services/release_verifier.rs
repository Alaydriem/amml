use std::io::{Cursor, Read};

use zip::ZipArchive;

use crate::errors::RegistryError;
use crate::services::{HashText, ListingCheck, ManifestReader, ReleaseSource};
use crate::structs::{AmmlRelease, ModRelease, ReleaseVersion, VerifiedMod};

/// The checks a release must pass before the registry lists it.
pub struct ReleaseVerifier<S: ReleaseSource> {
    source: S,
}

impl<S: ReleaseSource> ReleaseVerifier<S> {
    const SUMS: &'static str = "SHA256SUMS.txt";
    const HOST: &'static str = "https://github.com";
    // the largest single file the verifier reads out of a zip
    const MAX_ENTRY_BYTES: u64 = 4 * 1024 * 1024;

    pub fn new(source: S) -> Self {
        Self { source }
    }

    pub fn check_mod(
        &self,
        repo: &str,
        namespace: &str,
        name: &str,
        version: &str,
        now: &str,
    ) -> Result<VerifiedMod, RegistryError> {
        ReleaseVersion::parse(version)?;
        let asset = format!("{name}-{version}-windows-x64.zip");
        let (zip, sha256) = self.download(repo, version, &asset)?;
        let mut archive = Self::open(&zip)?;

        let folder = format!("@{namespace}/{name}/");
        Self::only_inside(&archive, &folder)?;
        let manifest_text = Self::read_text(&mut archive, &format!("{folder}amml.hcl"))?;
        let manifest = ManifestReader::parse(&manifest_text)?;
        Self::expect("namespace", namespace, &manifest.namespace)?;
        Self::expect("name", name, &manifest.name)?;
        Self::expect("version", version, &manifest.version)?;

        let mut listing = manifest.listing.ok_or_else(|| RegistryError::Listing {
            field: "listing",
            reason: "the amml.hcl has no listing block".to_string(),
        })?;
        ListingCheck::check(&listing)?;
        let icon = match listing.icon.take() {
            Some(path) => {
                let bytes = Self::read_bytes(&mut archive, &format!("{folder}{path}")).map_err(|_| {
                    RegistryError::Listing { field: "icon", reason: format!("`{path}` is not in the zip") }
                })?;
                ListingCheck::check_icon(&bytes)?;
                listing.icon = Some(format!("icons/@{namespace}/{name}.png"));
                Some(bytes)
            }
            None => None,
        };

        let release = ModRelease {
            version: version.to_string(),
            api_version: manifest.api_version,
            asset: format!("{}/{repo}/releases/download/{version}/{asset}", Self::HOST),
            sha256,
            size: zip.len() as u64,
            published: now.to_string(),
        };
        Ok(VerifiedMod { release, listing, icon })
    }

    pub fn check_amml(
        &self,
        repo: &str,
        version: &str,
        api_version: u32,
        game_versions: &[String],
    ) -> Result<AmmlRelease, RegistryError> {
        ReleaseVersion::parse(version)?;
        for game in game_versions {
            Self::game_version(game)?;
        }
        let asset = format!("amml-{version}-windows-x64.zip");
        let (zip, sha256) = self.download(repo, version, &asset)?;
        let mut archive = Self::open(&zip)?;

        for exe in ["amml.dll", "amml-patch.exe"] {
            let bytes =
                Self::read_bytes(&mut archive, exe).map_err(|_| Self::bad_zip(format!("`{exe}` is missing")))?;
            if !bytes.starts_with(b"MZ") {
                return Err(Self::bad_zip(format!("`{exe}` is not a Windows executable")));
            }
        }
        let has_recipe =
            Self::names(&archive)?.iter().any(|path| path.starts_with("amml/recipes/") && !path.ends_with('/'));
        if !has_recipe {
            return Err(Self::bad_zip("`amml/recipes/` has no files".to_string()));
        }
        let stamp = Self::read_text(&mut archive, "amml/amml-version.txt")?;
        Self::expect("amml-version.txt", version, stamp.trim_end())?;

        Ok(AmmlRelease {
            version: version.to_string(),
            api_version,
            game_versions: game_versions.to_vec(),
            asset: format!("{}/{repo}/releases/download/{version}/{asset}", Self::HOST),
            sha256,
            size: zip.len() as u64,
        })
    }

    // the zip and its SHA-256, after the SHA256SUMS.txt line for it agrees
    fn download(&self, repo: &str, tag: &str, asset: &str) -> Result<(Vec<u8>, String), RegistryError> {
        let sums = self.source.fetch(repo, tag, Self::SUMS)?;
        let zip = self.source.fetch(repo, tag, asset)?;
        let sums = String::from_utf8_lossy(&sums);
        let expected = sums
            .lines()
            .filter_map(|line| line.split_once(char::is_whitespace))
            .find(|(_, file)| file.trim().trim_start_matches('*') == asset)
            .map(|(hash, _)| hash.trim().to_ascii_lowercase())
            .ok_or_else(|| RegistryError::Missing(format!("{asset} in {}", Self::SUMS)))?;
        let actual = HashText::sha256(&zip);
        if actual != expected {
            return Err(RegistryError::HashMismatch { asset: asset.to_string() });
        }
        Ok((zip, actual))
    }

    // opens the zip and refuses any entry path that could leave the folder it is extracted into
    fn open(zip: &[u8]) -> Result<ZipArchive<Cursor<&[u8]>>, RegistryError> {
        let archive = ZipArchive::new(Cursor::new(zip)).map_err(|error| Self::bad_zip(error.to_string()))?;
        for path in Self::names(&archive)? {
            let body = path.strip_suffix('/').unwrap_or(&path);
            let unsafe_path = path.starts_with('/')
                || path.contains('\\')
                || path.contains(':')
                || body.split('/').any(|segment| segment.is_empty() || segment == ".." || segment == ".");
            if unsafe_path {
                return Err(Self::bad_zip(format!("`{path}` is not a safe entry path")));
            }
        }
        Ok(archive)
    }

    // every entry must sit inside the one folder `@<namespace>/<name>/`
    fn only_inside(archive: &ZipArchive<Cursor<&[u8]>>, folder: &str) -> Result<(), RegistryError> {
        for path in Self::names(archive)? {
            if !path.starts_with(folder) && path.trim_end_matches('/') != folder.trim_end_matches('/') {
                return Err(Self::bad_zip(format!("`{path}` is outside {folder}")));
            }
        }
        Ok(())
    }

    fn names(archive: &ZipArchive<Cursor<&[u8]>>) -> Result<Vec<String>, RegistryError> {
        archive
            .file_names()
            .map(|name| name.map(|text| text.into_owned()).map_err(|error| Self::bad_zip(error.to_string())))
            .collect()
    }

    fn read_bytes(archive: &mut ZipArchive<Cursor<&[u8]>>, path: &str) -> Result<Vec<u8>, RegistryError> {
        let entry = archive.by_name(path).map_err(|error| Self::bad_zip(format!("{path}: {error}")))?;
        let mut bytes = Vec::new();
        entry
            .take(Self::MAX_ENTRY_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| Self::bad_zip(format!("{path}: {error}")))?;
        if bytes.len() as u64 > Self::MAX_ENTRY_BYTES {
            return Err(Self::bad_zip(format!("{path} is larger than {} bytes", Self::MAX_ENTRY_BYTES)));
        }
        Ok(bytes)
    }

    fn read_text(archive: &mut ZipArchive<Cursor<&[u8]>>, path: &str) -> Result<String, RegistryError> {
        let bytes = Self::read_bytes(archive, path)?;
        String::from_utf8(bytes).map_err(|_| Self::bad_zip(format!("{path} is not UTF-8 text")))
    }

    fn expect(field: &'static str, expected: &str, got: &str) -> Result<(), RegistryError> {
        if expected == got {
            return Ok(());
        }
        Err(RegistryError::ManifestMismatch { field, expected: expected.to_string(), got: got.to_string() })
    }

    fn game_version(text: &str) -> Result<(), RegistryError> {
        let parts: Vec<&str> = text.split('.').collect();
        let numeric = parts.iter().all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()));
        if parts.len() == 4 && numeric {
            return Ok(());
        }
        Err(RegistryError::BadVersion(text.to_string()))
    }

    fn bad_zip(message: String) -> RegistryError {
        RegistryError::BadZip(message)
    }
}
