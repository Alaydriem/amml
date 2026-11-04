use amml_registry::{AmmlRelease, AmmlSection, ModDocument, RegistryError, RegistryUpdater, UpdateResult};

use crate::support::Samples;

const REPO: &str = "alaydriem/amml-zoom";

fn add(doc: &mut Option<ModDocument>, version: &str, sha: &str) -> Result<UpdateResult, RegistryError> {
    let allow = Samples::allow_list();
    let release = Samples::release(version, 2, sha, Samples::NOW);
    RegistryUpdater::add_mod(doc, &allow, REPO, "alaydriem", "zoom", release, &Samples::listing("Zoom"))
}

fn amml_release(version: &str, sha: &str) -> AmmlRelease {
    AmmlRelease {
        version: version.to_string(),
        api_version: 2,
        game_versions: vec!["1.26.60.5".to_string()],
        asset: format!("https://github.com/alaydriem/amml/releases/download/{version}/amml-{version}-windows-x64.zip"),
        sha256: sha.to_string(),
        size: 2000,
    }
}

fn empty_amml() -> AmmlSection {
    AmmlSection { repo: "alaydriem/amml".to_string(), versions: Vec::new() }
}

#[test]
fn adds_mod_newest_first() {
    let mut doc = None;
    let expected = [UpdateResult::Added, UpdateResult::Backfilled, UpdateResult::Added];
    for (version, result) in ["26.60.1", "26.60.0", "26.60.2"].into_iter().zip(expected) {
        assert_eq!(add(&mut doc, version, "aa").unwrap(), result);
    }
    let versions: Vec<_> = doc.unwrap().versions.into_iter().map(|release| release.version).collect();
    assert_eq!(versions, ["26.60.2", "26.60.1", "26.60.0"]);
}

#[test]
fn ten_is_newer_than_nine() {
    let mut doc = None;
    add(&mut doc, "26.60.9", "aa").unwrap();
    add(&mut doc, "26.60.10", "bb").unwrap();
    assert_eq!(doc.unwrap().versions[0].version, "26.60.10");
}

#[test]
fn same_version_same_hash_is_unchanged() {
    let mut doc = None;
    add(&mut doc, "26.60.1", "aa").unwrap();
    let before = doc.clone();
    assert_eq!(add(&mut doc, "26.60.1", "aa").unwrap(), UpdateResult::Unchanged);
    assert_eq!(doc, before);
}

#[test]
fn same_version_other_hash_is_conflict() {
    let mut doc = None;
    add(&mut doc, "26.60.1", "aa").unwrap();
    let before = doc.clone();
    match add(&mut doc, "26.60.1", "bb") {
        Err(RegistryError::Conflict { version }) => assert_eq!(version, "26.60.1"),
        other => panic!("{other:?}"),
    }
    assert_eq!(doc, before);
}

#[test]
fn mod_not_in_allow_list_is_refused() {
    let mut doc = None;
    let release = Samples::release("26.60.1", 2, "aa", Samples::NOW);
    let result = RegistryUpdater::add_mod(
        &mut doc,
        &Samples::allow_list(),
        REPO,
        "alaydriem",
        "nope",
        release,
        &Samples::listing("Nope"),
    );
    match result {
        Err(RegistryError::NotAllowed(id)) => assert_eq!(id, "@alaydriem/nope"),
        other => panic!("{other:?}"),
    }
    assert!(doc.is_none());
}

#[test]
fn repo_must_match_allow_list_ignoring_case() {
    let mut doc = None;
    let release = Samples::release("26.60.1", 2, "aa", Samples::NOW);
    let result = RegistryUpdater::add_mod(
        &mut doc,
        &Samples::allow_list(),
        "alaydriem/zoom",
        "alaydriem",
        "zoom",
        release.clone(),
        &Samples::listing("Zoom"),
    );
    assert!(matches!(result, Err(RegistryError::RepoMismatch { .. })));
    let upper = RegistryUpdater::add_mod(
        &mut doc,
        &Samples::allow_list(),
        "Alaydriem/AMML-Zoom",
        "alaydriem",
        "zoom",
        release,
        &Samples::listing("Zoom"),
    );
    assert_eq!(upper.unwrap(), UpdateResult::Added);
}

#[test]
fn metadata_follows_allow_list_and_newest_listing() {
    let mut doc = None;
    add(&mut doc, "26.60.1", "aa").unwrap();
    assert!(!doc.as_ref().unwrap().core);

    let mut allow = Samples::allow_list();
    allow.mods[0].core = true;
    allow.mods[0].default = false;
    let release = Samples::release("26.60.2", 2, "bb", Samples::NOW);
    let mut newer = Samples::listing("Zoom 2");
    newer.summary = "new text".to_string();
    RegistryUpdater::add_mod(&mut doc, &allow, REPO, "alaydriem", "zoom", release, &newer).unwrap();
    let current = doc.as_ref().unwrap();
    assert!(current.core);
    assert_eq!(current.listing.title, "Zoom 2");

    let old = Samples::release("26.60.0", 2, "cc", Samples::NOW);
    RegistryUpdater::add_mod(&mut doc, &allow, REPO, "alaydriem", "zoom", old, &Samples::listing("Zoom 0")).unwrap();
    assert_eq!(doc.unwrap().listing.title, "Zoom 2");
}

#[test]
fn mod_versions_are_bounded_and_oldest_drop() {
    let mut doc = None;
    for patch in 0..(RegistryUpdater::MAX_MOD_VERSIONS + 5) {
        add(&mut doc, &format!("26.60.{patch}"), "aa").unwrap();
    }
    let versions = doc.unwrap().versions;
    assert_eq!(versions.len(), RegistryUpdater::MAX_MOD_VERSIONS);
    assert_eq!(versions.last().unwrap().version, "26.60.5");
    assert_eq!(versions[0].version, format!("26.60.{}", RegistryUpdater::MAX_MOD_VERSIONS + 4));
}

#[test]
fn amml_release_keeps_game_versions_newest_first() {
    let allow = Samples::allow_list();
    let mut section = empty_amml();
    let first = amml_release("26.60.1", "aa");
    assert_eq!(RegistryUpdater::add_amml(&mut section, &allow, "alaydriem/amml", first).unwrap(), UpdateResult::Added);
    RegistryUpdater::add_amml(&mut section, &allow, "alaydriem/amml", amml_release("26.60.2", "bb")).unwrap();
    assert_eq!(section.versions[0].version, "26.60.2");
    assert_eq!(section.versions[1].game_versions, ["1.26.60.5"]);

    let again = RegistryUpdater::add_amml(&mut section, &allow, "alaydriem/amml", amml_release("26.60.2", "bb"));
    assert_eq!(again.unwrap(), UpdateResult::Unchanged);
    let clash = RegistryUpdater::add_amml(&mut section, &allow, "alaydriem/amml", amml_release("26.60.2", "zz"));
    assert!(matches!(clash, Err(RegistryError::Conflict { .. })));
    let other = RegistryUpdater::add_amml(&mut section, &allow, "alaydriem/other", amml_release("26.60.3", "cc"));
    assert!(matches!(other, Err(RegistryError::RepoMismatch { .. })));
}

#[test]
fn amml_versions_are_bounded() {
    let allow = Samples::allow_list();
    let mut section = empty_amml();
    for patch in 0..(RegistryUpdater::MAX_AMML_VERSIONS + 3) {
        let release = amml_release(&format!("26.60.{patch}"), "aa");
        RegistryUpdater::add_amml(&mut section, &allow, "alaydriem/amml", release).unwrap();
    }
    assert_eq!(section.versions.len(), RegistryUpdater::MAX_AMML_VERSIONS);
    assert_eq!(section.versions.last().unwrap().version, "26.60.3");
}

#[test]
fn a_mod_version_older_than_every_kept_version_is_unchanged() {
    let mut doc = None;
    for patch in 1..=RegistryUpdater::MAX_MOD_VERSIONS {
        add(&mut doc, &format!("26.60.{patch}"), "aa").unwrap();
    }
    let before = doc.clone();
    assert_eq!(add(&mut doc, "26.60.0", "bb").unwrap(), UpdateResult::Unchanged);
    assert_eq!(doc, before);
}

#[test]
fn an_amml_version_older_than_every_kept_version_is_unchanged() {
    let allow = Samples::allow_list();
    let mut section = empty_amml();
    for patch in 1..=RegistryUpdater::MAX_AMML_VERSIONS {
        let release = amml_release(&format!("26.60.{patch}"), "aa");
        RegistryUpdater::add_amml(&mut section, &allow, "alaydriem/amml", release).unwrap();
    }
    let before = section.clone();
    let old = amml_release("26.60.0", "bb");
    let result = RegistryUpdater::add_amml(&mut section, &allow, "alaydriem/amml", old).unwrap();
    assert_eq!(result, UpdateResult::Unchanged);
    assert_eq!(section, before);
}

#[test]
fn authorize_checks_the_allow_list_and_returns_its_spelling() {
    let allow = Samples::allow_list();
    let allowed = RegistryUpdater::authorize_mod(&allow, "ALAYDRIEM/amml-zoom", "alaydriem", "zoom").unwrap();
    assert_eq!(allowed.repo, "alaydriem/amml-zoom");
    assert!(matches!(
        RegistryUpdater::authorize_mod(&allow, REPO, "alaydriem", "nope"),
        Err(RegistryError::NotAllowed(_))
    ));
    assert!(matches!(
        RegistryUpdater::authorize_mod(&allow, "alaydriem/other", "alaydriem", "zoom"),
        Err(RegistryError::RepoMismatch { .. })
    ));
    assert_eq!(RegistryUpdater::authorize_amml(&allow, "Alaydriem/AMML").unwrap(), "alaydriem/amml");
    assert!(matches!(
        RegistryUpdater::authorize_amml(&allow, "alaydriem/other"),
        Err(RegistryError::RepoMismatch { .. })
    ));
}
