use amml_i18n_tool::app::{I18nToolApp, ToolCommand};
use tempfile::TempDir;

const POT: &str = "msgid \"\"\nmsgstr \"\"\n\"Plural-Forms: nplurals=2; plural=(n != 1);\\n\"\n\n\
msgid \"Save\"\nmsgstr \"\"\n\nmsgid \"Close\"\nmsgstr \"\"\n";

// The .po was not merged after "Close" joined the .pot: it is complete against itself
// but covers half of the current messages.
const STALE_PO: &str = "msgid \"\"\nmsgstr \"\"\n\"Plural-Forms: nplurals=2; plural=(n != 1);\\n\"\n\n\
msgid \"Save\"\nmsgstr \"Speichern\"\n";

// Review finding 4: the build gate must measure against the .pot, as the report does.
#[test]
fn build_measures_coverage_against_the_pot() {
    let directory = TempDir::new().unwrap_or_else(|error| panic!("{error}"));
    let i18n = directory.path();
    std::fs::write(i18n.join("amml-ui.pot"), POT).unwrap_or_else(|error| panic!("{error}"));
    std::fs::write(i18n.join("de.po"), STALE_PO).unwrap_or_else(|error| panic!("{error}"));
    let packs = i18n.join("packs");

    I18nToolApp::execute(&ToolCommand::Build { po_dir: i18n.to_path_buf(), out_dir: packs.clone() })
        .unwrap_or_else(|error| panic!("{error}"));

    assert!(!packs.join("de.json").exists(), "a locale at 50 % of the .pot must not ship");
}
