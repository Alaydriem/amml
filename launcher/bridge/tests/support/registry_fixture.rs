use super::FakeHttp;
use amml_launcher_core::{RegistryCache, RegistryClient, RegistryRef};
use amml_registry::{AmmlSection, JsonText, Listing, ModDocument, ModRelease, TreeBuilder};
use std::path::Path;

// A registry tree built with the registry crate's own builder and served from memory.
pub struct RegistryFixture;

impl RegistryFixture {
    pub const NOW: &'static str = "2026-11-02T10:00:00Z";

    pub fn document(namespace: &str, name: &str, version: &str, size: u64, published: &str, api: u32) -> ModDocument {
        ModDocument {
            schema: 2,
            id: format!("@{namespace}/{name}"),
            namespace: namespace.to_string(),
            name: name.to_string(),
            repo: format!("{namespace}/{name}"),
            core: false,
            default: false,
            listing: Listing {
                title: name.to_string(),
                summary: format!("{name} summary"),
                description: Some(format!("{name} description")),
                authors: vec!["Alaydriem".to_string()],
                license: "BSD-3-Clause".to_string(),
                source: format!("https://github.com/{namespace}/{name}"),
                homepage: None,
                tags: vec!["camera".to_string()],
                icon: Some(format!("icons/{name}.png")),
            },
            versions: vec![ModRelease {
                version: version.to_string(),
                api_version: api,
                asset: format!("https://example.test/{name}.zip"),
                sha256: "ab".repeat(32),
                size,
                published: published.to_string(),
            }],
        }
    }

    // `count` mods named `<prefix>00`, `<prefix>01`, ... under the namespace `ns`, all for API 2
    pub fn many(prefix: &str, count: usize) -> Vec<ModDocument> {
        (0..count)
            .map(|index| {
                let published = format!("2026-10-{:02}T10:00:00Z", 1 + index % 28);
                Self::document("ns", &format!("{prefix}{index:04}"), "1.0.0", 1000 + index as u64, &published, 2)
            })
            .collect()
    }

    // serves the registry at `base` and returns a client for it
    pub fn serve(base: &str, documents: &[ModDocument], cache: &Path) -> (RegistryClient<FakeHttp>, FakeHttp) {
        let http = FakeHttp::new();
        let amml = AmmlSection { repo: "alaydriem/amml".to_string(), versions: Vec::new() };
        let tree = TreeBuilder::build("Test registry", Self::NOW, &amml, documents).unwrap();
        for (path, bytes) in tree.files {
            http.put(format!("{base}{path}"), bytes);
        }
        for document in documents {
            let path = format!("{base}mods/@{}/{}.json", document.namespace, document.name);
            http.put(path, JsonText::compact(document).unwrap());
        }
        let client = RegistryClient::new(
            RegistryRef::new(base.to_string()),
            http.clone(),
            RegistryCache::new(cache.to_path_buf(), RegistryCache::BUDGET),
        );
        (client, http)
    }
}
