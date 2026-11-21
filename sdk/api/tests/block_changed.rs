use std::sync::{Arc, Mutex};

use amml_abi::BlockChanged;
use amml_api::testing::FakeGame;
use amml_api::{BlockChangedExt, BlockPos, BlockTypeId, HookOrder};

#[test]
fn ext_reads_dimension_pos_and_types() {
    let fake = FakeGame::new();
    fake.offer::<BlockChanged>();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<BlockChanged>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push((event.dimension(), event.pos(), event.layer(), event.new(), event.old()));
        })
        .unwrap();
    let pos = BlockPos { x: -3, y: 64, z: 17 };
    let dispatch = fake.raise_block_changed(1, pos, 0, BlockTypeId(7), BlockTypeId(2));
    assert!(!dispatch.panicked);
    // A change inside one type is raised too.
    fake.raise_block_changed(1, pos, 1, BlockTypeId(7), BlockTypeId(7));
    assert_eq!(
        *seen.lock().unwrap(),
        vec![(1, pos, 0, BlockTypeId(7), BlockTypeId(2)), (1, pos, 1, BlockTypeId(7), BlockTypeId(7)),]
    );
}
