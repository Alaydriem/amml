use crate::errors::RegistryError;
use crate::structs::{
    AllowList, AllowedMod, AmmlRelease, AmmlSection, Listing, ModDocument, ModRelease, ReleaseVersion, UpdateResult,
};

/// The update rules: what a new release does to the AMML section or to one mod document.
pub struct RegistryUpdater;

impl RegistryUpdater {
    pub const MAX_AMML_VERSIONS: usize = 50;
    pub const MAX_MOD_VERSIONS: usize = 100;
    pub const SCHEMA: u32 = 2;

    pub fn add_amml(
        section: &mut AmmlSection,
        allow: &AllowList,
        repo: &str,
        release: AmmlRelease,
    ) -> Result<UpdateResult, RegistryError> {
        Self::authorize_amml(allow, repo)?;
        let version = ReleaseVersion::parse(&release.version)?;
        if let Some(known) = section.versions.iter().find(|known| known.version == release.version) {
            return Self::duplicate(&known.sha256, &release.sha256, &release.version);
        }
        let mut keyed = Vec::with_capacity(section.versions.len() + 1);
        for known in section.versions.drain(..) {
            keyed.push((ReleaseVersion::parse(&known.version)?, known));
        }
        let added = release.version.clone();
        keyed.push((version, release));
        keyed.sort_by_key(|entry| std::cmp::Reverse(entry.0));
        keyed.truncate(Self::MAX_AMML_VERSIONS);
        if !keyed.iter().any(|(_, kept)| kept.version == added) {
            section.versions = keyed.into_iter().map(|(_, release)| release).collect();
            return Ok(UpdateResult::Unchanged);
        }
        section.repo = allow.amml_repo.clone();
        section.versions = keyed.into_iter().map(|(_, release)| release).collect();
        Ok(UpdateResult::Added)
    }

    pub fn add_mod(
        document: &mut Option<ModDocument>,
        allow: &AllowList,
        repo: &str,
        namespace: &str,
        name: &str,
        release: ModRelease,
        listing: &Listing,
    ) -> Result<UpdateResult, RegistryError> {
        let allowed = Self::authorize_mod(allow, repo, namespace, name)?;
        let version = ReleaseVersion::parse(&release.version)?;
        if let Some(known) =
            document.as_ref().and_then(|doc| doc.versions.iter().find(|v| v.version == release.version))
        {
            return Self::duplicate(&known.sha256, &release.sha256, &release.version);
        }

        let mut keyed = Vec::new();
        let mut newest = None;
        if let Some(existing) = document.as_ref() {
            for known in &existing.versions {
                let parsed = ReleaseVersion::parse(&known.version)?;
                newest = newest.max(Some(parsed));
                keyed.push((parsed, known.clone()));
            }
        }
        let is_newest = newest.is_none_or(|current| version > current);
        let previous_listing = document.as_ref().map(|doc| doc.listing.clone());
        let added = release.version.clone();
        keyed.push((version, release));
        keyed.sort_by_key(|entry| std::cmp::Reverse(entry.0));
        keyed.truncate(Self::MAX_MOD_VERSIONS);
        if !keyed.iter().any(|(_, kept)| kept.version == added) {
            return Ok(UpdateResult::Unchanged);
        }

        *document = Some(ModDocument {
            schema: Self::SCHEMA,
            id: allowed.id(),
            namespace: allowed.namespace.clone(),
            name: allowed.name.clone(),
            repo: allowed.repo.clone(),
            core: allowed.core,
            default: allowed.default || allowed.core,
            listing: match previous_listing {
                Some(previous) if !is_newest => previous,
                _ => listing.clone(),
            },
            versions: keyed.into_iter().map(|(_, release)| release).collect(),
        });
        Ok(if is_newest { UpdateResult::Added } else { UpdateResult::Backfilled })
    }

    /// Checks the mod and the repo against the allow list before anything is downloaded.
    pub fn authorize_mod<'a>(
        allow: &'a AllowList,
        repo: &str,
        namespace: &str,
        name: &str,
    ) -> Result<&'a AllowedMod, RegistryError> {
        let allowed =
            allow.find(namespace, name).ok_or_else(|| RegistryError::NotAllowed(format!("@{namespace}/{name}")))?;
        Self::same_repo(&allowed.repo, repo)?;
        Ok(allowed)
    }

    /// Checks the AMML repo against the allow list before anything is downloaded.
    pub fn authorize_amml<'a>(allow: &'a AllowList, repo: &str) -> Result<&'a str, RegistryError> {
        Self::same_repo(&allow.amml_repo, repo)?;
        Ok(&allow.amml_repo)
    }

    /// Makes the policy fields of a document agree with the allow list. A mod that is not in the list is left alone.
    pub fn sync_policy(document: &mut ModDocument, allow: &AllowList) {
        if let Some(allowed) = allow.find(&document.namespace, &document.name) {
            document.repo = allowed.repo.clone();
            document.core = allowed.core;
            document.default = allowed.default || allowed.core;
        }
    }

    fn same_repo(expected: &str, got: &str) -> Result<(), RegistryError> {
        if expected.eq_ignore_ascii_case(got) {
            return Ok(());
        }
        Err(RegistryError::RepoMismatch { expected: expected.to_string(), got: got.to_string() })
    }

    fn duplicate(known: &str, new: &str, version: &str) -> Result<UpdateResult, RegistryError> {
        if known.eq_ignore_ascii_case(new) {
            return Ok(UpdateResult::Unchanged);
        }
        Err(RegistryError::Conflict { version: version.to_string() })
    }
}
