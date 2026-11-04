use crate::support::Samples;
use amml_launcher_bridge::{NameCheck, NameProblemView};

#[test]
fn a_bad_instance_name_gives_a_name_problem() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    for name in [".hidden", "..", "a/b", "CON", "a\\b", " ", "a b", "x."] {
        assert!(NameCheck::problem(name, &[], &paths).is_some(), "{name:?}");
    }
    assert_eq!(NameCheck::problem("", &[], &paths), Some(NameProblemView::Empty));
    assert_eq!(NameCheck::problem("  ", &[], &paths), Some(NameProblemView::Empty));
    assert_eq!(NameCheck::problem("..", &[], &paths), Some(NameProblemView::Bad));
    assert_eq!(NameCheck::problem("1.26.60.05", &["1.26.60.05".into()], &paths), Some(NameProblemView::Taken));
}

#[test]
fn a_taken_name_matches_without_regard_to_case_and_a_free_name_has_no_problem() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Samples::paths(dir.path());
    let taken = ["Play".to_string()];
    assert_eq!(NameCheck::problem("play", &taken, &paths), Some(NameProblemView::Taken));
    assert_eq!(NameCheck::problem("1.26.60.05-b", &taken, &paths), None);
}
