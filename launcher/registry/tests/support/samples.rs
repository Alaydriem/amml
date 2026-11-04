use amml_registry::{AllowList, AllowedMod, Listing, ModDocument, ModRelease};

/// Small builders for the values that many tests share.
pub struct Samples;

impl Samples {
    pub const NOW: &'static str = "2026-11-02T10:00:00Z";

    pub fn listing(title: &str) -> Listing {
        Listing {
            title: title.to_string(),
            summary: format!("{title} summary"),
            description: None,
            authors: vec!["Charles R. Portwood II".to_string()],
            license: "BSD-3-Clause".to_string(),
            source: "https://github.com/alaydriem/amml-zoom".to_string(),
            homepage: None,
            tags: vec!["camera".to_string()],
            icon: None,
        }
    }

    pub fn release(version: &str, api_version: u32, sha256: &str, published: &str) -> ModRelease {
        ModRelease {
            version: version.to_string(),
            api_version,
            asset: format!(
                "https://github.com/alaydriem/amml-zoom/releases/download/{version}/zoom-{version}-windows-x64.zip"
            ),
            sha256: sha256.to_string(),
            size: 1000,
            published: published.to_string(),
        }
    }

    pub fn allowed(namespace: &str, name: &str, repo: &str) -> AllowedMod {
        AllowedMod {
            namespace: namespace.to_string(),
            name: name.to_string(),
            repo: repo.to_string(),
            core: false,
            default: true,
        }
    }

    pub fn allow_list() -> AllowList {
        AllowList {
            amml_repo: "alaydriem/amml".to_string(),
            mods: vec![Self::allowed("alaydriem", "zoom", "alaydriem/amml-zoom")],
        }
    }

    pub fn document(namespace: &str, name: &str, releases: Vec<ModRelease>) -> ModDocument {
        ModDocument {
            schema: 2,
            id: format!("@{namespace}/{name}"),
            namespace: namespace.to_string(),
            name: name.to_string(),
            repo: format!("alaydriem/amml-{name}"),
            core: false,
            default: true,
            listing: Self::listing(name),
            versions: releases,
        }
    }
}
