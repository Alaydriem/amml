use std::sync::atomic::{AtomicU8, Ordering};

use amml_abi::{AmmlApi, AmmlModInfo, AmmlStatus, ClientTick};
use amml_api::testing::FakeGame;
use amml_api::{AmmlMod, ApiError, Ctx, DisableCtx, ModGlue, Uses};

use crate::support::ShortTable;

static ALIVE_AT_DISABLE: AtomicU8 = AtomicU8::new(9);

struct Demo;

impl AmmlMod for Demo {
    const NAMESPACE: &'static str = "acme";
    const NAME: &'static str = "demo";
    const USES: Uses = amml_api::amml_uses! { feature "core" { required ClientTick; } };

    fn enable(ctx: &Ctx) -> Result<(), ApiError> {
        ctx.log().info("enabled", &[]);
        Ok(())
    }

    fn disable(_ctx: &Ctx, how: &DisableCtx) {
        ALIVE_AT_DISABLE.store(u8::from(how.game_alive), Ordering::SeqCst);
    }
}

#[test]
fn load_writes_a_v3_block_with_the_uses_rows() {
    let fake = FakeGame::new();
    let mut info = std::mem::MaybeUninit::<AmmlModInfo>::zeroed();
    // SAFETY: the table lives in `fake` and `info` is writable for one AmmlModInfo.
    let status = unsafe { ModGlue::load::<Demo>(fake.api_table(), info.as_mut_ptr()) };
    assert_eq!(status, AmmlStatus::OK);
    // SAFETY: load wrote the block on OK.
    let info = unsafe { info.assume_init() };
    assert_eq!(info.size, AmmlModInfo::SIZE_V3);
    assert_eq!(info.api_version, 3);
    // SAFETY: the uses block and its rows are static.
    let uses = unsafe { &*AmmlModInfo::uses(&raw const info).unwrap() };
    assert_eq!(uses.count, 1);

    assert_eq!(ModGlue::enable::<Demo>(), AmmlStatus::OK);
    assert_eq!(fake.logs(), vec!["info enabled".to_string()]);
}

#[test]
fn disable_tells_the_mod_whether_the_game_runs() {
    let fake = FakeGame::new();
    fake.open_gate();
    let mut info = std::mem::MaybeUninit::<AmmlModInfo>::zeroed();
    // SAFETY: the table lives in `fake` and `info` is writable for one AmmlModInfo.
    let status = unsafe { ModGlue::load::<Demo>(fake.api_table(), info.as_mut_ptr()) };
    assert_eq!(status, AmmlStatus::OK);

    assert_eq!(ModGlue::disable::<Demo>(), AmmlStatus::OK);
    assert_eq!(ALIVE_AT_DISABLE.load(Ordering::SeqCst), 1);
}

#[test]
fn a_v2_loader_table_is_refused_without_writing_the_info_block() {
    let table = ShortTable::table(AmmlApi::SIZE_V1, 2);
    let mut info = std::mem::MaybeUninit::<AmmlModInfo>::uninit();
    let bytes = info.as_mut_ptr().cast::<u8>();
    let size = std::mem::size_of::<AmmlModInfo>();
    // SAFETY: `info` is writable for `size` bytes.
    unsafe { bytes.write_bytes(0xAB, size) };

    // SAFETY: the table is live and `info` is writable for one AmmlModInfo.
    let status = unsafe { ModGlue::load::<Demo>(&raw const table, info.as_mut_ptr()) };

    assert_eq!(status, AmmlStatus::VERSION_MISMATCH);
    // SAFETY: every byte was written above.
    let after = unsafe { std::slice::from_raw_parts(bytes, size) };
    assert!(after.iter().all(|byte| *byte == 0xAB));
}
