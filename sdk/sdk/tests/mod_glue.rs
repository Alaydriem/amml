use amml_abi::{AmmlModInfo, AmmlStatus};
use amml_sdk::{AmmlMod, Api, ModGlue, SdkError};

use crate::support::{FakeLoader, LINES};

struct Demo;

impl AmmlMod for Demo {
    const NAMESPACE: &'static str = "acme";
    const NAME: &'static str = "demo";

    fn enable(_api: &Api) -> Result<(), SdkError> {
        Ok(())
    }

    fn disable(api: &Api) {
        let _ = api;
    }
}

#[test]
fn load_reports_the_namespace_and_a_v2_block() {
    let table = FakeLoader::current();
    let mut info = std::mem::MaybeUninit::<AmmlModInfo>::uninit();
    // SAFETY: the table is live and `info` is writable for one AmmlModInfo.
    let status = unsafe { ModGlue::load::<Demo>(&raw const table, info.as_mut_ptr()) };
    assert_eq!(status, AmmlStatus::OK);
    // SAFETY: load wrote the block on OK.
    let info = unsafe { info.assume_init() };
    assert_eq!(info.size, 40);
    // SAFETY: the strings are static.
    assert_eq!(unsafe { info.name.as_str() }.unwrap(), "demo");
    // SAFETY: the strings are static.
    assert_eq!(unsafe { info.namespace.as_str() }.unwrap(), "acme");

    assert_eq!(ModGlue::disable::<Demo>(), AmmlStatus::OK);
    ModGlue::unload::<Demo>();
    amml_sdk::ModRuntime::get().unwrap().logger().info("hello");
    let lines = LINES.lock().unwrap();
    assert!(lines.iter().any(|line| line.starts_with("@acme/demo ")), "{lines:?}");
}

#[test]
fn load_writes_only_the_v2_prefix_of_the_info_block() {
    let table = FakeLoader::current();
    let mut buffer = [0xABu8; 64];
    // SAFETY: the buffer is 64 bytes, larger than AmmlModInfo; it is 8-aligned through the cast below.
    let aligned = buffer.as_mut_ptr().align_offset(8);
    // SAFETY: `aligned` bytes keep the pointer inside the 64-byte buffer, and 48 bytes remain.
    let info = unsafe { buffer.as_mut_ptr().add(aligned) };
    // SAFETY: the table is live and `info` is writable for at least the 40-byte v2 prefix.
    let status = unsafe { ModGlue::load::<Demo>(&raw const table, info.cast::<AmmlModInfo>()) };
    assert_eq!(status, AmmlStatus::OK);
    // SAFETY: bytes 40..48 of the block lie within the buffer.
    let tail = unsafe { std::slice::from_raw_parts(info.add(40), 8) };
    assert_eq!(tail, &[0xAB; 8]);
}
