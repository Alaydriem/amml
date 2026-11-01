use amml_i18n::message::Domain;

// A mod declares its own domain. amml-i18n keeps no list of mods.
const MY_MOD: Domain = Domain::new("my-mod-2");

#[test]
fn a_domain_is_named_by_the_mod_that_declares_it() {
    assert_eq!(MY_MOD.name(), "my-mod-2");
}

#[test]
fn two_domains_with_the_same_name_are_the_same_domain() {
    assert_eq!(Domain::new("amml-ui"), Domain::new("amml-ui"));
    assert_ne!(Domain::new("amml-ui"), MY_MOD);
}

// The name is the .pot file stem and a folder name, so it must be safe on every file system.
#[test]
#[should_panic(expected = "domain name")]
fn an_empty_name_is_rejected() {
    let _ = Domain::new(std::hint::black_box(""));
}

#[test]
#[should_panic(expected = "domain name")]
fn a_name_with_capitals_or_spaces_is_rejected() {
    let _ = Domain::new(std::hint::black_box("My Mod"));
}

#[test]
#[should_panic(expected = "domain name")]
fn a_name_with_a_path_separator_is_rejected() {
    let _ = Domain::new(std::hint::black_box("../amml-ui"));
}
