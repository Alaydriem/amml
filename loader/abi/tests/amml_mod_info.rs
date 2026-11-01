use amml_abi::{AmmlModInfo, AmmlStr};

fn info(size: u32, api_version: u32) -> AmmlModInfo {
    AmmlModInfo { size, api_version, name: AmmlStr::EMPTY }
}

#[test]
fn loader_accepts_a_mod_asking_for_its_version_or_older() {
    assert!(info(AmmlModInfo::SIZE_V1, 1).accepted_by(1));
    assert!(info(AmmlModInfo::SIZE_V1, 1).accepted_by(2));
}

#[test]
fn loader_refuses_a_mod_asking_for_a_newer_api() {
    assert!(!info(AmmlModInfo::SIZE_V1, 2).accepted_by(1));
}

#[test]
fn loader_refuses_an_info_block_shorter_than_v1() {
    assert!(!info(AmmlModInfo::SIZE_V1 - 1, 1).accepted_by(1));
}

#[test]
fn header_reads_the_two_leading_fields_and_refuses_null() {
    let block = info(AmmlModInfo::SIZE_V1, 3);
    // SAFETY: `block` is a live, aligned AmmlModInfo.
    let header = unsafe { AmmlModInfo::header(&raw const block) };
    assert_eq!(header, Some((AmmlModInfo::SIZE_V1, 3)));
    // SAFETY: a null pointer is allowed and is not read.
    assert_eq!(unsafe { AmmlModInfo::header(core::ptr::null()) }, None);
}
