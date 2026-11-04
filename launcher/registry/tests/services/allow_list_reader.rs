use amml_registry::{AllowListReader, RegistryError};

const SAMPLE: &str = r#"
amml { repo = "alaydriem/amml" }

mod "amml" "modmanager" {
  repo = "alaydriem/amml-modmanager"
  core = true
}

mod "alaydriem" "zoom" {
  repo    = "alaydriem/amml-zoom"
  default = true
}

mod "alaydriem" "extra" {
  repo = "alaydriem/amml-extra"
}
"#;

#[test]
fn reads_amml_and_mods() {
    let list = AllowListReader::parse(SAMPLE).unwrap();
    assert_eq!(list.amml_repo, "alaydriem/amml");
    assert_eq!(list.mods.len(), 3);
    let zoom = list.find("alaydriem", "zoom").unwrap();
    assert_eq!(zoom.id(), "@alaydriem/zoom");
    assert_eq!(zoom.repo, "alaydriem/amml-zoom");
    assert!(zoom.default && !zoom.core);
    let extra = list.find("alaydriem", "extra").unwrap();
    assert!(!extra.default && !extra.core);
}

#[test]
fn core_implies_default() {
    let list = AllowListReader::parse(SAMPLE).unwrap();
    let manager = list.find("amml", "modmanager").unwrap();
    assert!(manager.core);
    assert!(manager.default);
}

#[test]
fn duplicate_mod_is_bad_allow_list() {
    let text = format!("{SAMPLE}\nmod \"alaydriem\" \"zoom\" {{\n  repo = \"alaydriem/amml-zoom\"\n}}\n");
    assert!(matches!(AllowListReader::parse(&text), Err(RegistryError::BadAllowList(_))));
}

#[test]
fn missing_repo_and_unknown_keys_are_refused() {
    let no_repo = "amml { repo = \"alaydriem/amml\" }\nmod \"a\" \"b\" {\n  default = true\n}\n";
    assert!(matches!(AllowListReader::parse(no_repo), Err(RegistryError::BadAllowList(_))));
    let old_key = "amml { repo = \"alaydriem/amml\" }\nmod \"a\" \"b\" {\n  repo = \"a/b\"\n  description = \"x\"\n}\n";
    assert!(matches!(AllowListReader::parse(old_key), Err(RegistryError::BadAllowList(_))));
    assert!(matches!(
        AllowListReader::parse("mod \"a\" \"b\" {\n  repo = \"a/b\"\n}\n"),
        Err(RegistryError::BadAllowList(_))
    ));
}
