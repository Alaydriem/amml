use super::{FakeServer, Route};
use amml_registry::{
    AmmlRelease, AmmlSection, FileRef, HashText, IndexSummary, JsonText, Listing, ModDocument, ModRelease,
    PublishedTree, RegistryRoot, TreeBuilder,
};
use std::collections::BTreeMap;

// Schema 2 registry trees, built with the registry crate's own builder and served from a fake server.
pub struct RegistryTree;

impl RegistryTree {
    pub const NOW: &'static str = "2026-11-02T10:00:00Z";

    pub fn amml_release(version: &str, api: u32, games: &[&str]) -> AmmlRelease {
        AmmlRelease {
            version: version.to_string(),
            api_version: api,
            game_versions: games.iter().map(|game| game.to_string()).collect(),
            asset: "https://example.test/amml.zip".to_string(),
            sha256: "ab".to_string(),
            size: 5,
        }
    }

    pub fn amml(releases: Vec<AmmlRelease>) -> AmmlSection {
        AmmlSection { repo: "alaydriem/amml".to_string(), versions: releases }
    }

    pub fn release(version: &str, api: u32, asset: &str, sha256: &str, size: u64) -> ModRelease {
        ModRelease {
            version: version.to_string(),
            api_version: api,
            asset: asset.to_string(),
            sha256: sha256.to_string(),
            size,
            published: Self::NOW.to_string(),
            min_amml: None,
        }
    }

    pub fn release_min(version: &str, api: u32, min_amml: Option<&str>) -> ModRelease {
        ModRelease { min_amml: min_amml.map(str::to_string), ..Self::release(version, api, "u", "h", 1) }
    }

    pub fn listing(title: &str, tags: &[&str]) -> Listing {
        Listing {
            title: title.to_string(),
            summary: format!("{title} summary"),
            description: None,
            authors: vec!["Charles R. Portwood II".to_string()],
            license: "BSD-3-Clause".to_string(),
            source: "https://github.com/alaydriem/amml".to_string(),
            homepage: None,
            tags: tags.iter().map(|tag| tag.to_string()).collect(),
            icon: None,
        }
    }

    pub fn document(namespace: &str, name: &str, core: bool, default: bool, releases: Vec<ModRelease>) -> ModDocument {
        ModDocument {
            schema: 2,
            id: format!("@{namespace}/{name}"),
            namespace: namespace.to_string(),
            name: name.to_string(),
            repo: format!("alaydriem/{name}"),
            core,
            default,
            listing: Self::listing(&Self::title(name), &["camera"]),
            versions: releases,
        }
    }

    pub fn title(name: &str) -> String {
        let mut chars = name.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        }
    }

    pub fn build(name: &str, amml: &AmmlSection, documents: &[ModDocument]) -> PublishedTree {
        let mut tree = TreeBuilder::build(name, Self::NOW, amml, documents).unwrap();
        for document in documents {
            tree.files.insert(
                format!("mods/@{}/{}.json", document.namespace, document.name),
                JsonText::pretty(document).unwrap(),
            );
        }
        tree
    }

    // every file of the tree becomes a route with an etag of its own hash
    pub fn serve(server: &FakeServer, tree: &PublishedTree) {
        for (path, bytes) in &tree.files {
            let etag = format!("\"{}\"", &HashText::sha256(bytes)[..16]);
            server.replace(&format!("/{path}"), Route::with_etag(bytes.clone(), &etag));
        }
    }

    pub fn publish(server: &FakeServer, name: &str, amml: &AmmlSection, documents: &[ModDocument]) -> PublishedTree {
        let tree = Self::build(name, amml, documents);
        Self::serve(server, &tree);
        tree
    }

    pub fn root(amml: AmmlSection) -> RegistryRoot {
        RegistryRoot {
            schema: 2,
            name: "Official".to_string(),
            published: Self::NOW.to_string(),
            mod_count: 0,
            amml,
            index: IndexSummary { page_size: 100, apis: BTreeMap::new() },
            names: FileRef { path: "names.json".to_string(), sha256: String::new(), size: 0 },
        }
    }
}
