use std::path::Path;

use amml_i18n_tool::catalog::CatalogCheck;
use amml_ui::AmmlUiMessage;

#[test]
fn message_catalog() {
    let problems = CatalogCheck::problems::<AmmlUiMessage>();
    assert!(problems.is_empty(), "{}", problems.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n"));
}

#[test]
fn pot_up_to_date() {
    let i18n = Path::new(env!("CARGO_MANIFEST_DIR")).join("i18n");
    let current = CatalogCheck::pot_is_current::<AmmlUiMessage>(&i18n).unwrap_or_else(|error| panic!("{error}"));
    assert!(current, "amml-ui.pot is out of date: run `mise run i18n:extract`");
}
