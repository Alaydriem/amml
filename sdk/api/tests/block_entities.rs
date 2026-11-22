use std::sync::{Arc, Mutex};

use amml_abi::BlockEntityPass;
use amml_api::testing::FakeGame;
use amml_api::{
    ApiError, Area, BlockEntityCamera, BlockEntityGhost, BlockEntityPassExt, BlockHandle, BlockPos, Face,
    GhostEntityOptions, HookOrder, SkullPose,
};

// What the pass listener saw: the camera and the result of its two draws, one per dispatch.
type Seen = Arc<Mutex<Vec<(BlockEntityCamera, Result<(), ApiError>)>>>;

const CHEST: u64 = 5;
const SIGN: u64 = 6;

// A fake with the pass and both areas offered, a world joined, and the chest and sign kinds named.
fn fake() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer::<BlockEntityPass>();
    fake.offer_area(Area::RENDER_BLOCK_ENTITIES);
    fake.offer_area(Area::RENDER_CHEST_PAIR);
    fake.open_gate();
    fake.set_epoch(1);
    fake.set_block_entity_kind(BlockHandle::for_test(CHEST), "chest");
    fake.set_block_entity_kind(BlockHandle::for_test(SIGN), "sign");
    fake
}

#[test]
fn create_and_draw_from_the_fake() {
    let fake = fake();
    let blocks = fake.ctx().block_entities();
    let chest = blocks
        .create(
            BlockHandle::for_test(CHEST),
            Some(&[10, 0, 0, 0]),
            GhostEntityOptions { chest_pair: Some(BlockPos { x: 0, y: 0, z: 1 }), skull: None },
        )
        .unwrap();
    let sign = blocks.create(BlockHandle::for_test(SIGN), None, GhostEntityOptions::default()).unwrap();
    let ghosts = Arc::new(Mutex::new(Some((chest, sign))));
    let seen: Seen = Arc::new(Mutex::new(Vec::new()));
    let (held, record) = (ghosts.clone(), seen.clone());
    let listener = fake
        .ctx()
        .events()
        .listen::<BlockEntityPass>(HookOrder::NORMAL, move |event, _| {
            let held = held.lock().unwrap();
            let (chest, sign) = held.as_ref().unwrap();
            let first = event.draw_block_entity(chest, BlockPos { x: 3, y: 65, z: -4 });
            let second = event.draw_block_entity(sign, BlockPos { x: 4, y: 65, z: -4 });
            record.lock().unwrap().push((event.camera(), first.and(second)));
        })
        .unwrap();

    fake.raise_block_entity_pass(BlockEntityCamera::Player);
    fake.raise_block_entity_pass(BlockEntityCamera::Other);

    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen, [(BlockEntityCamera::Player, Ok(())), (BlockEntityCamera::Other, Ok(()))]);
    assert_eq!(
        fake.block_entities().calls(),
        [
            "create kind=chest nbt=4 pair=0,0,1",
            "create kind=sign nbt=0",
            "draw 1 3,65,-4",
            "draw 2 4,65,-4",
            "draw 1 3,65,-4",
            "draw 2 4,65,-4",
        ]
    );
    drop(listener);
    let (chest, _sign) = ghosts.lock().unwrap().take().unwrap();
    blocks.free(chest);
    assert_eq!(fake.block_entities().live(), 1);
}

#[test]
fn draw_outside_the_pass_is_refused_and_a_freed_ghost_is_stale() {
    let fake = fake();
    let blocks = fake.ctx().block_entities();
    let kept: Arc<Mutex<Option<BlockEntityPass>>> = Arc::new(Mutex::new(None));
    let copy = kept.clone();
    let listener = fake
        .ctx()
        .events()
        .listen::<BlockEntityPass>(HookOrder::NORMAL, move |event, _| *copy.lock().unwrap() = Some(*event))
        .unwrap();
    fake.raise_block_entity_pass(BlockEntityCamera::Player);
    let ghost = blocks.create(BlockHandle::for_test(SIGN), None, GhostEntityOptions::default()).unwrap();

    let event = kept.lock().unwrap().unwrap();
    assert_eq!(event.draw_block_entity(&ghost, BlockPos { x: 0, y: 0, z: 0 }), Err(ApiError::Refused));
    drop(listener);
    fake.leave_block_entity_world();
    // The world leave freed the ghost: nothing is live, and a draw in a later pass gives `Stale`.
    assert_eq!(fake.block_entities().live(), 0);
    let held = Arc::new(Mutex::new(Some(ghost)));
    let result: Arc<Mutex<Option<Result<(), ApiError>>>> = Arc::new(Mutex::new(None));
    let (inner, record) = (held.clone(), result.clone());
    let _listener = fake
        .ctx()
        .events()
        .listen::<BlockEntityPass>(HookOrder::NORMAL, move |event, _| {
            let ghost = inner.lock().unwrap();
            *record.lock().unwrap() =
                Some(event.draw_block_entity(ghost.as_ref().unwrap(), BlockPos { x: 0, y: 0, z: 0 }));
        })
        .unwrap();
    fake.raise_block_entity_pass(BlockEntityCamera::Player);
    assert_eq!(*result.lock().unwrap(), Some(Err(ApiError::Stale)));
}

#[test]
fn options_need_their_areas() {
    let fake = FakeGame::new();
    fake.offer_area(Area::RENDER_BLOCK_ENTITIES);
    fake.open_gate();
    fake.set_epoch(1);
    let blocks = fake.ctx().block_entities();

    let pair = blocks.create(
        BlockHandle::for_test(CHEST),
        None,
        GhostEntityOptions { chest_pair: Some(BlockPos { x: 1, y: 0, z: 0 }), skull: None },
    );
    let skull = blocks.create(
        BlockHandle::for_test(CHEST),
        None,
        GhostEntityOptions { chest_pair: None, skull: Some(SkullPose { face: Face::Up, turn: 4 }) },
    );
    let far = blocks.create(
        BlockHandle::for_test(CHEST),
        None,
        GhostEntityOptions { chest_pair: Some(BlockPos { x: 2, y: 0, z: 0 }), skull: None },
    );

    assert_eq!(pair.map(|_| ()), Err(ApiError::NotAvailable));
    assert_eq!(skull.map(|_| ()), Err(ApiError::NotAvailable));
    assert_eq!(far.map(|_| ()), Err(ApiError::Loader(amml_abi::AmmlStatus::INVALID_ARGUMENT)));
    assert_eq!(BlockEntityGhost::MAX_LIVE, 256);
}

#[test]
fn skull_pose_needs_its_area() {
    const SKULL: u64 = 7;
    let fake = FakeGame::new();
    fake.offer_area(Area::RENDER_BLOCK_ENTITIES);
    fake.open_gate();
    fake.set_epoch(1);
    fake.set_block_entity_kind(BlockHandle::for_test(SKULL), "skull");
    fake.set_block_entity_kind(BlockHandle::for_test(CHEST), "chest");
    let blocks = fake.ctx().block_entities();
    let pose = |face, turn| GhostEntityOptions { chest_pair: None, skull: Some(SkullPose { face, turn }) };

    let before = blocks.create(BlockHandle::for_test(SKULL), None, pose(Face::Up, 4));
    fake.offer_area(Area::RENDER_SKULL_POSE);
    let made = blocks.create(BlockHandle::for_test(SKULL), None, pose(Face::Up, 4));
    let on_chest = blocks.create(BlockHandle::for_test(CHEST), None, pose(Face::North, 0));
    let past_turn = blocks.create(BlockHandle::for_test(SKULL), None, pose(Face::Up, 16));
    let unknown_face = blocks.create(BlockHandle::for_test(SKULL), None, pose(Face::Other(6), 0));

    let invalid = Err(ApiError::Loader(amml_abi::AmmlStatus::INVALID_ARGUMENT));
    assert_eq!(before.map(|_| ()), Err(ApiError::NotAvailable));
    assert!(made.is_ok());
    assert_eq!(on_chest.map(|_| ()), invalid);
    assert_eq!(past_turn.map(|_| ()), invalid);
    assert_eq!(unknown_face.map(|_| ()), invalid);
    assert_eq!(fake.block_entities().calls(), ["create kind=skull nbt=0 skull=1,4"]);
}
