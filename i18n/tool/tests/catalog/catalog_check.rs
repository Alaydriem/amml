use amml_i18n_tool::catalog::{CatalogCheck, CatalogProblem};
use amml_i18n_tool::pot::PotWriter;
use tempfile::TempDir;

use crate::fixture::{BrokenMessage, SampleMessage};

#[test]
fn the_broken_fixture_reports_each_problem_once() {
    let problems = CatalogCheck::problems::<BrokenMessage>();

    assert_eq!(
        problems,
        vec![
            CatalogProblem::EmptyMsgid { index: 0 },
            CatalogProblem::RepeatedKey { index: 2, msgid: "Save" },
            CatalogProblem::PlaceholderMismatch { index: 3, msgid: "{count} file" },
            CatalogProblem::IndexOutOfPlace { position: 4, index: 9 },
        ]
    );
}

#[test]
fn a_problem_names_its_place_and_text() {
    let problem = CatalogProblem::RepeatedKey { index: 2, msgid: "Save" };
    assert_eq!(problem.to_string(), "message 2 (\"Save\") repeats the msgid and context of an earlier message");
}

#[test]
fn the_sample_fixture_has_no_problems() {
    assert!(CatalogCheck::problems::<SampleMessage>().is_empty());
}

#[test]
fn a_stale_pot_is_not_current() {
    let dir = TempDir::new().unwrap_or_else(|error| panic!("{error}"));
    std::fs::write(dir.path().join("amml-ui.pot"), "msgid \"old\"\nmsgstr \"\"\n")
        .unwrap_or_else(|error| panic!("{error}"));

    let current = CatalogCheck::pot_is_current::<SampleMessage>(dir.path()).unwrap_or_else(|error| panic!("{error}"));

    assert!(!current);
}

#[test]
fn a_missing_pot_is_not_current() {
    let dir = TempDir::new().unwrap_or_else(|error| panic!("{error}"));

    let current = CatalogCheck::pot_is_current::<SampleMessage>(dir.path()).unwrap_or_else(|error| panic!("{error}"));

    assert!(!current);
}

#[test]
fn a_fresh_pot_is_current() {
    let dir = TempDir::new().unwrap_or_else(|error| panic!("{error}"));
    std::fs::write(dir.path().join("amml-ui.pot"), PotWriter::render::<SampleMessage>())
        .unwrap_or_else(|error| panic!("{error}"));

    let current = CatalogCheck::pot_is_current::<SampleMessage>(dir.path()).unwrap_or_else(|error| panic!("{error}"));

    assert!(current);
}
