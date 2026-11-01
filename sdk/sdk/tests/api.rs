use amml_abi::{AMML_API_VERSION, AmmlApi, AmmlStatus};
use amml_sdk::{Api, SdkError};

use crate::support::FakeLoader;

#[test]
fn a_table_shorter_than_v1_or_older_than_the_mod_needs_is_refused() {
    let short = FakeLoader::table(AmmlApi::SIZE_V1 - 1, AMML_API_VERSION);
    // SAFETY: the table is live; the SDK reads only the two leading fields before refusing it.
    let result = unsafe { Api::from_raw(&raw const short, AMML_API_VERSION) };
    assert!(matches!(result, Err(SdkError::TableTooSmall(_))));

    let old = FakeLoader::table(AmmlApi::SIZE_V1, 0);
    // SAFETY: the table is live and full size.
    let result = unsafe { Api::from_raw(&raw const old, AMML_API_VERSION) };
    assert!(matches!(result, Err(SdkError::LoaderTooOld(0))));

    // SAFETY: a null table is allowed and is not read.
    let result = unsafe { Api::from_raw(std::ptr::null(), AMML_API_VERSION) };
    assert!(matches!(result, Err(SdkError::NullApi)));
}

#[test]
fn resolve_gives_the_value_and_names_a_missing_entry() {
    let table = FakeLoader::current();
    // SAFETY: the table is live and full size.
    let api = unsafe { Api::from_raw(&raw const table, AMML_API_VERSION) }.unwrap();

    assert_eq!(api.resolve("known").unwrap(), 0x1234);
    let error = api.resolve("absent").unwrap_err();
    assert!(matches!(&error, SdkError::MissingEntry(id) if id == "absent"));
    assert_eq!(error.status(), AmmlStatus::MISSING_ENTRY);
}

#[test]
fn the_mods_own_version_decides_whether_a_loader_table_is_old_enough() {
    let v1 = FakeLoader::table(AmmlApi::SIZE_V1, 1);

    // SAFETY: the table is live and full size.
    assert!(unsafe { Api::from_raw(&raw const v1, 1) }.is_ok());
    // SAFETY: as above.
    assert!(matches!(unsafe { Api::from_raw(&raw const v1, 2) }, Err(SdkError::LoaderTooOld(1))));
    let v3 = FakeLoader::table(AmmlApi::SIZE_V1, 3);
    // SAFETY: the table is live and full size.
    assert!(unsafe { Api::from_raw(&raw const v3, 1) }.is_ok());
}
