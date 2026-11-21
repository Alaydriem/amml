use std::sync::{Arc, Mutex};

use amml_abi::{AmmlStr, ProviderGone, ProviderReady};
use amml_api::testing::FakeGame;
use amml_api::{HookOrder, Provided, ProviderGoneExt, ProviderReadyExt};

#[repr(C)]
struct TestTable {
    size: u32,
    version: u32,
    answer: u32,
}

// SAFETY: `TestTable` is `#[repr(C)]` and starts with `size, version`.
#[allow(unsafe_code)]
unsafe impl Provided for TestTable {
    const NAME: &'static str = "test.table";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 2;
}

static TABLE: TestTable = TestTable { size: std::mem::size_of::<TestTable>() as u32, version: 2, answer: 42 };

#[test]
fn require_returns_the_provided_table() {
    let fake = FakeGame::new();
    let ctx = fake.ctx();
    ctx.registry().provide(&TABLE).unwrap();

    let table = ctx.registry().require::<TestTable>().unwrap();

    assert!(std::ptr::eq(table, &TABLE));
    assert_eq!(table.version, 2);
    assert_eq!(table.answer, 42);
}

#[test]
fn require_without_a_provider_gives_none() {
    let fake = FakeGame::new();
    assert!(fake.ctx().registry().require::<TestTable>().is_none());
}

#[test]
fn provider_events_give_the_table_name_without_unsafe_code() {
    let fake = FakeGame::new();
    fake.offer::<ProviderReady>();
    fake.offer::<ProviderGone>();
    let names = Arc::new(Mutex::new(Vec::new()));
    let ready = names.clone();
    let _ready = fake
        .ctx()
        .events()
        .listen::<ProviderReady>(HookOrder::NORMAL, move |event, _| {
            ready.lock().unwrap().push(event.name().map(str::to_string));
        })
        .unwrap();
    let gone = names.clone();
    let _gone = fake
        .ctx()
        .events()
        .listen::<ProviderGone>(HookOrder::NORMAL, move |event, _| {
            gone.lock().unwrap().push(event.name().map(str::to_string));
        })
        .unwrap();
    let header = FakeGame::header::<ProviderReady>();
    let name = AmmlStr::from_str("test.table");
    fake.raise(&mut ProviderReady { header, name, major: 1, minor: 0 });
    let header = FakeGame::header::<ProviderGone>();
    let bad = AmmlStr { ptr: core::ptr::null(), len: 3 };
    fake.raise(&mut ProviderGone { header, name: bad, major: 1 });

    assert_eq!(*names.lock().unwrap(), [Some("test.table".to_string()), None]);
}
