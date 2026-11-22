use std::sync::{Arc, Mutex};

use amml_abi::BlockChanged;
use amml_api::testing::{FakeClient, FakeGame, FakeWorld};
use amml_api::{Area, BlockChangedExt, BlockHandle, BlockPos, BlockTypeId, HookOrder, Screen};

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

#[test]
fn new_state_from_the_fake_event() {
    let fake = FakeGame::new();
    fake.offer::<BlockChanged>();
    fake.offer_area(Area::WORLD_BLOCKS);
    fake.open_gate();
    fake.set_epoch(1);
    fake.set_client(FakeClient {
        local_player: true,
        world_active: true,
        top_screen: Screen::Other,
        world: Some(FakeWorld::default()),
        ..FakeClient::default()
    });
    let ctx = fake.ctx();
    let inside = Arc::new(Mutex::new(Vec::new()));
    let record = inside.clone();
    let kept = Arc::new(Mutex::new(None));
    let copy = kept.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<BlockChanged>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push(event.new_state(&ctx));
            *copy.lock().unwrap() = Some(*event);
        })
        .unwrap();
    let pos = BlockPos { x: 1, y: 2, z: 3 };
    let new_state = BlockHandle::for_test(42);

    fake.raise_block_changed_v2(1, pos, 0, BlockTypeId(7), BlockTypeId(2), Some(new_state));
    fake.raise_block_changed_v2(1, pos, 0, BlockTypeId(7), BlockTypeId(2), None);

    assert_eq!(*inside.lock().unwrap(), vec![Some(new_state), None]);
    // A copy of the event kept past the call names no state.
    let copy = kept.lock().unwrap().take().unwrap();
    assert_eq!(copy.new_state(&fake.ctx()), None);
}
