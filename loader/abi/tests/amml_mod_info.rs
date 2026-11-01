use amml_abi::{AMML_API_VERSION, AmmlModInfo, AmmlStr};

fn info(size: u32, api_version: u32) -> AmmlModInfo {
    AmmlModInfo { size, api_version, name: AmmlStr::EMPTY, namespace: AmmlStr::EMPTY }
}

#[test]
fn a_v2_block_is_40_bytes_and_a_v1_block_is_not_accepted() {
    assert_eq!(std::mem::size_of::<AmmlModInfo>(), 40);
    let block = AmmlModInfo {
        size: AmmlModInfo::SIZE_V1,
        api_version: 1,
        name: AmmlStr::from_str("x"),
        namespace: AmmlStr::from_str("y"),
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
