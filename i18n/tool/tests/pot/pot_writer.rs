use amml_i18n_tool::po::PoReader;
use amml_i18n_tool::pot::PotWriter;
use tempfile::TempDir;

use crate::fixture::SampleMessage;

#[test]
fn pot_is_deterministic() {
    assert_eq!(PotWriter::render::<SampleMessage>(), PotWriter::render::<SampleMessage>());
}

#[test]
fn pot_has_context_and_plural() {
    let pot = PotWriter::render::<SampleMessage>();
    assert!(pot.contains("msgctxt \"audio\"\nmsgid \"Output\"\nmsgstr \"\"\n"), "{pot}");
    assert!(
        pot.contains(
            "msgid \"{count} player nearby\"\nmsgid_plural \"{count} players nearby\"\nmsgstr[0] \"\"\nmsgstr[1] \"\"\n"
        ),
        "{pot}"
    );
    assert!(pot.contains("#. The audio device that plays sound\n"), "{pot}");
    assert!(pot.contains("\"X-Domain: amml-ui\\n\""), "{pot}");
}

#[test]
fn placeholder_messages_are_flagged_for_translation_tools() {
    let pot = PotWriter::render::<SampleMessage>();
    assert!(pot.contains("#, python-brace-format\nmsgid \"Hello, {name}\""), "{pot}");
}

#[test]
fn entries_follow_the_enum_order() {
    let pot = PotWriter::render::<SampleMessage>();
    let sign_in = pot.find("Sign In Again").unwrap_or(usize::MAX);
    let greeting = pot.find("Hello, {name}").unwrap_or(0);
    assert!(sign_in < greeting);
}

#[test]
fn the_pot_reads_back_with_every_message() {
    let catalog = PoReader::parse(&PotWriter::render::<SampleMessage>()).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(catalog.messages.len(), 5);
    assert!(catalog.messages.iter().all(|message| !message.is_translated()));
}

#[test]
fn quotes_backslashes_and_newlines_are_escaped() {
    assert_eq!(PotWriter::escape("a \"b\" \\ c\nd\te"), "a \\\"b\\\" \\\\ c\\nd\\te");
}

#[test]
fn write_reports_a_change_and_check_mode_leaves_the_file_alone() {
    let directory = TempDir::new().unwrap_or_else(|error| panic!("{error}"));
    let path = directory.path().join("i18n").join("amml-ui.pot");

    let stale = PotWriter::write::<SampleMessage>(&path, true).unwrap_or_else(|error| panic!("{error}"));
    assert!(stale, "a missing pot is stale");
    assert!(!path.exists(), "check mode must not write");

    assert!(PotWriter::write::<SampleMessage>(&path, false).unwrap_or_else(|error| panic!("{error}")));
    assert!(!PotWriter::write::<SampleMessage>(&path, false).unwrap_or_else(|error| panic!("{error}")));
    assert!(!PotWriter::write::<SampleMessage>(&path, true).unwrap_or_else(|error| panic!("{error}")));
}

// Review finding 5: git with core.autocrlf checks a .pot out with CRLF line ends.
#[test]
fn a_pot_checked_out_with_crlf_line_ends_is_not_stale() {
    let directory = TempDir::new().unwrap_or_else(|error| panic!("{error}"));
    let path = directory.path().join("amml-ui.pot");
    std::fs::write(&path, PotWriter::render::<SampleMessage>().replace('\n', "\r\n"))
        .unwrap_or_else(|error| panic!("{error}"));
    assert!(!PotWriter::write::<SampleMessage>(&path, true).unwrap_or_else(|error| panic!("{error}")));
}
