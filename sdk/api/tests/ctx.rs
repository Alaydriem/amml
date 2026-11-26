use std::ffi::c_void;

use amml_abi::{AmmlApi, AmmlServices, AmmlStatus, ClientTick};
use amml_api::testing::FakeGame;
use amml_api::{ApiError, Ctx, HookOrder, Uses};

use crate::support::ShortTable;

#[test]
fn a_v2_loader_table_is_refused() {
    let table = ShortTable::table(AmmlApi::SIZE_V1, 2);
    // SAFETY: the table is live and `size` bytes are readable.
    let refused = unsafe { Ctx::from_raw(&raw const table, "@acme/demo", Uses::NONE) };
    assert!(matches!(refused, Err(ApiError::Loader(AmmlStatus::VERSION_MISMATCH))));
}

#[test]
fn a_short_table_that_claims_version_3_is_refused() {
    let table = ShortTable::table(AmmlApi::SIZE_V1, 3);
    // SAFETY: the table is live and `size` bytes are readable.
    let refused = unsafe { Ctx::from_raw(&raw const table, "@acme/demo", Uses::NONE) };
    assert!(matches!(refused, Err(ApiError::Loader(AmmlStatus::VERSION_MISMATCH))));
}

/// A services table of a later version: the v1 fields, then one more table pointer.
#[repr(C)]
struct GrownServices {
    base: AmmlServices,
    area: *const c_void,
}

#[test]
fn a_services_table_longer_than_v1_is_accepted() {
    let fake = FakeGame::new();
    fake.offer::<ClientTick>();
    // SAFETY: the fake's tables are full v3 tables that live while `fake` does.
    let api = unsafe { *fake.api_table() };
    // SAFETY: as above; `services` is non-null in the fake's table.
    let services = unsafe { *api.services };
    let grown = GrownServices {
        base: AmmlServices { size: std::mem::size_of::<GrownServices>() as u32, version: 2, ..services },
        area: std::ptr::null(),
    };
    let api = AmmlApi { services: &raw const grown.base, ..api };

    // SAFETY: `api` and `grown` live for the test, and `size` bytes of each are readable.
    let ctx = unsafe { Ctx::from_raw(&raw const api, "@acme/demo", Uses::NONE) }.unwrap();

    assert!(ctx.events().listen::<ClientTick>(HookOrder::NORMAL, |_, _| {}).is_ok());
    assert!(grown.area.is_null());
}
