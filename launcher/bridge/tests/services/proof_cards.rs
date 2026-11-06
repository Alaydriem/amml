use amml_launcher_bridge::{CardStateView, CardView, CatalogService, NeedsAmmlView};
use amml_launcher_core::{
    AmmlLevel, CacheLock, RegistryCache, RegistryClient, RegistryRef, RegistrySet, UreqHttpFetch,
};
use amml_proof_fixture::ProofTree;
use amml_registry::ReleaseVersion;

struct Rig {
    // keeps the server and the tree alive
    proof: ProofTree,
    set: RegistrySet<UreqHttpFetch>,
    registry_id: String,
    _work: tempfile::TempDir,
    _cache: tempfile::TempDir,
}

impl Rig {
    fn new() -> Rig {
        let work = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        let proof = ProofTree::build(work.path());
        let registry = RegistryRef::official_at(format!("{}/", proof.registry_base));
        let registry_id = registry.cache_key();
        let client = RegistryClient::new(
            registry,
            UreqHttpFetch::new(UreqHttpFetch::agent()),
            RegistryCache::new(cache.path().to_path_buf(), RegistryCache::BUDGET, CacheLock::new()),
        );
        let mut set = RegistrySet::new(vec![client]);
        set.refresh_all();
        Rig { proof, set, registry_id, _work: work, _cache: cache }
    }

    fn level(version: &str) -> AmmlLevel {
        AmmlLevel::Release(ReleaseVersion::parse(version).unwrap())
    }

    fn cards(&mut self, amml: &str) -> Vec<CardView> {
        CatalogService::page(&mut self.set, &self.registry_id, 2, &Self::level(amml), 1, &[]).unwrap()
    }

    fn card<'a>(cards: &'a [CardView], name: &str) -> &'a CardView {
        let id = format!("@proof/{name}");
        cards.iter().find(|card| card.id == id).unwrap_or_else(|| panic!("no card for {id}"))
    }
}

fn needs(version: &str, game: Option<&str>) -> Option<NeedsAmmlView> {
    Some(NeedsAmmlView { version: version.to_string(), game: game.map(str::to_string) })
}

#[test]
fn cards_on_old() {
    let mut rig = Rig::new();
    let cell = |mod_cell: &str| format!("{} / {mod_cell} / browse", ProofTree::OLD_AMML);

    let cards = rig.cards(ProofTree::OLD_AMML);

    let plain = Rig::card(&cards, "plain");
    assert_eq!(plain.state, CardStateView::Available, "{}", cell("plain 1.1.0"));
    assert_eq!(plain.needs_amml, None, "{}", cell("plain 1.1.0"));

    let gated = Rig::card(&cards, "gated");
    assert_eq!(gated.state, CardStateView::Available, "{}", cell("gated 1.0.0"));
    assert_eq!(gated.version, "1.0.0", "{}", cell("gated 1.0.0"));
    assert_eq!(gated.needs_amml, needs("26.60.6", None), "{}", cell("gated 2.0.0 note"));

    let onlygated = Rig::card(&cards, "onlygated");
    assert_eq!(onlygated.state, CardStateView::NeedsAmml, "{}", cell("onlygated 1.0.0"));
    assert_eq!(
        onlygated.needs_amml,
        needs(ProofTree::NEW_AMML, Some(ProofTree::NEW_GAME)),
        "{}",
        cell("onlygated 1.0.0")
    );
}

#[test]
fn cards_on_new() {
    let mut rig = Rig::new();

    let cards = rig.cards(ProofTree::NEW_AMML);

    for (name, version) in [("plain", "1.1.0"), ("gated", "2.0.0"), ("onlygated", "1.0.0")] {
        let card = Rig::card(&cards, name);
        let cell = format!("{} / {name} {version} / browse", ProofTree::NEW_AMML);
        assert_eq!(card.state, CardStateView::Available, "{cell}");
        assert_eq!(card.needs_amml, None, "{cell}");
        assert_eq!(card.version, version, "{cell}");
    }
}

#[test]
fn detail_on_old() {
    let mut rig = Rig::new();
    let level = Rig::level(ProofTree::OLD_AMML);
    let cell = format!("{} / gated 1.0.0 / detail", ProofTree::OLD_AMML);

    let detail = CatalogService::detail(&mut rig.set, &rig.registry_id, "@proof/gated", 2, &level).unwrap();

    assert_eq!(detail.card.state, CardStateView::Available, "{cell}");
    assert_eq!(detail.card.version, "1.0.0", "{cell}");
    assert_eq!(detail.card.needs_amml, needs("26.60.6", None), "{cell}");
}

#[test]
fn cards_make_no_document_requests() {
    let mut rig = Rig::new();

    rig.cards(ProofTree::OLD_AMML);

    let requests = rig.proof.server.requests();
    assert!(!requests.is_empty(), "browse / no requests were recorded");
    let documents: Vec<&String> = requests.iter().filter(|path| path.contains("/mods/@proof/")).collect();
    assert!(documents.is_empty(), "{} / browse / document fetches: {documents:?}", ProofTree::OLD_AMML);
}
