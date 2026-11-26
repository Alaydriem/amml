use amml_abi::{AMML_API_VERSION, AmmlModInfo, AmmlStr, AmmlUses};

fn info(size: u32, api_version: u32) -> AmmlModInfo {
    AmmlModInfo { size, api_version, name: AmmlStr::EMPTY, namespace: AmmlStr::EMPTY, uses: core::ptr::null() }
}

#[test]
fn a_v2_block_is_40_bytes_and_a_v1_block_is_not_accepted() {
    assert_eq!(AmmlModInfo::SIZE_V2, 40);
    let block = AmmlModInfo {
        size: AmmlModInfo::SIZE_V1,
        api_version: 1,
        name: AmmlStr::from_str("x"),
        namespace: AmmlStr::from_str("y"),
        uses: core::ptr::null(),
    };
    assert!(!block.accepted_by(AMML_API_VERSION));
    let block = AmmlModInfo { size: AmmlModInfo::SIZE_V2, ..block };
    assert!(block.accepted_by(2));
}

#[test]
fn loader_accepts_a_mod_asking_for_its_version_or_older() {
    assert!(info(AmmlModInfo::SIZE_V2, 1).accepted_by(2));
    assert!(info(AmmlModInfo::SIZE_V2, 2).accepted_by(2));
}

#[test]
fn loader_refuses_a_mod_asking_for_a_newer_api() {
    assert!(!info(AmmlModInfo::SIZE_V2, 3).accepted_by(2));
}

#[test]
fn loader_refuses_an_info_block_shorter_than_v2() {
    assert!(!info(AmmlModInfo::SIZE_V2 - 1, 2).accepted_by(2));
}

#[test]
fn header_reads_the_two_leading_fields_and_refuses_null() {
    let block = info(AmmlModInfo::SIZE_V2, 3);
    // SAFETY: `block` is a live, aligned AmmlModInfo.
    let header = unsafe { AmmlModInfo::header(&raw const block) };
    assert_eq!(header, Some((AmmlModInfo::SIZE_V2, 3)));
    // SAFETY: a null pointer is allowed and is not read.
    assert_eq!(unsafe { AmmlModInfo::header(core::ptr::null()) }, None);
}

#[test]
fn a_v2_info_block_has_no_uses() {
    let block = info(AmmlModInfo::SIZE_V2, 2);
    // SAFETY: `block` is a live, aligned AmmlModInfo.
    assert!(unsafe { AmmlModInfo::uses(&raw const block) }.is_none());
}

#[test]
fn a_v3_info_block_gives_its_uses_pointer() {
    let uses = AmmlUses { size: AmmlUses::SIZE_V1, version: 1, rows: core::ptr::null(), count: 0 };
    let block = AmmlModInfo { size: AmmlModInfo::SIZE_V3, uses: &raw const uses, ..info(AmmlModInfo::SIZE_V3, 3) };
    // SAFETY: `block` is a live, aligned AmmlModInfo of size SIZE_V3.
    assert_eq!(unsafe { AmmlModInfo::uses(&raw const block) }, Some(&raw const uses));
}

#[test]
fn a_v3_info_block_with_null_uses_has_none() {
    let block = info(AmmlModInfo::SIZE_V3, 3);
    // SAFETY: `block` is a live, aligned AmmlModInfo of size SIZE_V3.
    assert!(unsafe { AmmlModInfo::uses(&raw const block) }.is_none());
}

#[test]
fn uses_of_a_null_block_is_none() {
    // SAFETY: a null pointer is checked before any read.
    assert!(unsafe { AmmlModInfo::uses(core::ptr::null()) }.is_none());
}
