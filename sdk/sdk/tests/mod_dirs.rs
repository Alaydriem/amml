use std::path::PathBuf;

use amml_abi::AMML_API_VERSION;
use amml_sdk::{Api, ModDirs};

use crate::support::FakeLoader;

fn dirs_of(module: &str) -> Option<ModDirs> {
    let table = FakeLoader::current();
    // SAFETY: the table is live and full size.
    let api = unsafe { Api::from_raw(&raw const table, AMML_API_VERSION) }.unwrap();
    ModDirs::new(&api, module).ok()
}

#[test]
fn the_shared_data_folder_is_set_when_the_loader_answers_kind_3() {
    let dirs = dirs_of("shared").unwrap();

    assert_eq!(dirs.shared_data_dir, Some(PathBuf::from(r"C:\s")));
}

#[test]
fn an_old_loader_that_refuses_kind_3_leaves_the_shared_data_folder_unset() {
    let dirs = dirs_of("old").unwrap();

    assert_eq!(dirs.shared_data_dir, None);
    assert_eq!(dirs.data_dir, PathBuf::from(r"C:\m"));
}

#[test]
fn any_other_kind_3_failure_also_leaves_it_unset() {
    assert_eq!(dirs_of("broken").unwrap().shared_data_dir, None);
}

#[test]
fn an_empty_kind_3_answer_leaves_it_unset() {
    let dirs = dirs_of("rootless").unwrap();

    assert_eq!(dirs.shared_data_dir, None);
    assert_eq!(dirs.data_dir, PathBuf::from(r"C:\m"));
}

#[test]
fn a_failure_of_kinds_0_to_2_still_fails() {
    assert!(dirs_of("nobody").is_none());
}
