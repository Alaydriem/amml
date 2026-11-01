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
