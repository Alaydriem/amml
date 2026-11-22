use std::sync::{Arc, Mutex};

use amml_abi::{AmmlStatus, WorldRendered};
use amml_api::testing::{FakeCanvas, FakeGame};
use amml_api::{
    ApiError, Area, BlockGrid, BlockHandle, BlockMesher, BlockPos, HookOrder, MeshBuilt, MesherOptions,
    WorldRenderedExt,
};

// The results of the builds the listener made, one per dispatch.
type Results = Arc<Mutex<Vec<Result<MeshBuilt, ApiError>>>>;

fn listening(fake: &FakeGame, grid: (Vec<u16>, [u32; 3]), palette: Vec<BlockHandle>) -> Results {
    let results: Results = Arc::new(Mutex::new(Vec::new()));
    let record = results.clone();
    let (cells, dims) = grid;
    let listener = fake
        .ctx()
        .events()
        .listen::<WorldRendered>(HookOrder::NORMAL, move |event, _| {
            let Some(mesher) = event.mesher() else {
                record.lock().unwrap().push(Err(ApiError::NotAvailable));
                return;
            };
            let grid = BlockGrid { dims, origin: BlockPos { x: 4, y: 64, z: -2 }, cells: &cells, palette: &palette };
            let options = MesherOptions { alpha: 0x80, skip_matching_world: false, water_tint: false };
            record.lock().unwrap().push(mesher.build(&grid, &options));
        })
        .unwrap();
    std::mem::forget(listener);
    results
}

#[test]
fn mesher_from_the_fake_returns_a_mesh() {
    let fake = FakeGame::new();
    fake.offer::<WorldRendered>();
    fake.offer_area(Area::RENDER_WORLD_CANVAS.at(3));
    fake.offer_area(Area::RENDER_BLOCK_MESHER);
    // A 3 x 3 x 3 grid: one inner cell with block 0, a border cell with block 1.
    let mut cells = vec![BlockGrid::EMPTY; 27];
    cells[13] = 0;
    cells[0] = 1;
    let results = listening(&fake, (cells, [3, 3, 3]), vec![BlockHandle::for_test(5), BlockHandle::for_test(6)]);
    let canvas = FakeCanvas::default();

    fake.raise_world_rendered(&canvas);
    fake.raise_world_rendered(&canvas);

    let results = results.lock().unwrap();
    let first = results[0].as_ref().unwrap();
    let second = results[1].as_ref().unwrap();
    // The fake draws the one inner cell as six quads; each build gives a mesh of its own.
    assert_eq!((first.quads, first.skipped.clone()), (6, vec![]));
    assert_ne!(first.mesh, second.mesh);
    assert_eq!(*canvas.calls.borrow(), ["mesher build cells=1 quads=6", "mesher build cells=1 quads=6"]);
    assert_eq!(fake.live_meshes(), 2);
}

#[test]
fn mesher_refuses_a_grid_over_the_window_and_cells_past_the_palette() {
    let fake = FakeGame::new();
    fake.offer::<WorldRendered>();
    fake.offer_area(Area::RENDER_WORLD_CANVAS.at(3));
    fake.offer_area(Area::RENDER_BLOCK_MESHER);
    let wide = listening(&fake, (vec![BlockGrid::EMPTY; 21], [21, 1, 1]), vec![]);
    let past = listening(&fake, (vec![2; 27], [3, 3, 3]), vec![BlockHandle::for_test(5)]);

    fake.raise_world_rendered(&FakeCanvas::default());

    assert_eq!(BlockMesher::MAX_DIM, 20);
    assert_eq!(wide.lock().unwrap()[0].as_ref().unwrap_err(), &ApiError::Refused);
    assert_eq!(past.lock().unwrap()[0].as_ref().unwrap_err(), &ApiError::Loader(AmmlStatus::INVALID_ARGUMENT));
}

#[test]
fn mesher_needs_the_area() {
    let fake = FakeGame::new();
    fake.offer::<WorldRendered>();
    fake.offer_area(Area::RENDER_WORLD_CANVAS.at(3));
    let results = listening(&fake, (vec![0; 27], [3, 3, 3]), vec![BlockHandle::for_test(5)]);

    fake.raise_world_rendered(&FakeCanvas::default());

    assert_eq!(results.lock().unwrap()[0].as_ref().unwrap_err(), &ApiError::NotAvailable);
}

// What a listener that opens a group, appends the same grid twice and finishes it saw.
#[derive(Debug)]
struct GroupRun {
    begun: Result<(), ApiError>,
    appended: Vec<Result<(), ApiError>>,
    built: Result<MeshBuilt, ApiError>,
}

type GroupResults = Arc<Mutex<Vec<GroupRun>>>;

fn grouping(fake: &FakeGame, slot: [u32; 3], appends: u32) -> GroupResults {
    let results: GroupResults = Arc::new(Mutex::new(Vec::new()));
    let record = results.clone();
    let listener = fake
        .ctx()
        .events()
        .listen::<WorldRendered>(HookOrder::NORMAL, move |event, _| {
            let Some(mesher) = event.mesher() else { return };
            // A 3 x 3 x 3 grid with one inner cell.
            let mut cells = vec![BlockGrid::EMPTY; 27];
            cells[13] = 0;
            let palette = [BlockHandle::for_test(5)];
            let grid = BlockGrid {
                dims: [3, 3, 3],
                origin: BlockPos { x: 4, y: 64, z: -2 },
                cells: &cells,
                palette: &palette,
            };
            let options = MesherOptions { alpha: 0xFF, skip_matching_world: false, water_tint: false };
            let mut group = match mesher.begin_group(slot) {
                Ok(group) => group,
                Err(error) => {
                    record.lock().unwrap().push(GroupRun { begun: Err(error), appended: vec![], built: Err(error) });
                    return;
                }
            };
            let appended = (0..appends)
                .map(|index| mesher.append(&mut group, &grid, BlockPos { x: 16 * index as i32, y: 0, z: 0 }, &options))
                .collect();
            let built = mesher.finish(group);
            record.lock().unwrap().push(GroupRun { begun: Ok(()), appended, built });
        })
        .unwrap();
    std::mem::forget(listener);
    results
}

#[test]
fn group_from_the_fake() {
    let fake = FakeGame::new();
    fake.offer::<WorldRendered>();
    fake.offer_area(Area::RENDER_WORLD_CANVAS.at(3));
    fake.offer_area(Area::RENDER_BLOCK_MESHER);
    let results = grouping(&fake, [16, 16, 16], 2);
    let canvas = FakeCanvas::default();

    fake.raise_world_rendered(&canvas);

    let results = results.lock().unwrap();
    let run = &results[0];
    assert_eq!(run.begun, Ok(()));
    assert_eq!(run.appended, vec![Ok(()), Ok(())]);
    // Two appends of one inner cell each: twelve quads in one mesh of the fake mod.
    let built = run.built.as_ref().unwrap();
    assert_eq!((built.quads, built.skipped.clone()), (12, vec![]));
    assert_eq!(
        *canvas.calls.borrow(),
        [
            "mesher group begin slot=16,16,16",
            "mesher group append cells=1 quads=6 offset=0,0,0",
            "mesher group append cells=1 quads=6 offset=16,0,0",
            "mesher group finish quads=12",
        ]
    );
    assert_eq!(fake.live_meshes(), 1);
}

#[test]
fn a_group_slot_of_no_size_is_an_invalid_argument_in_the_fake() {
    let fake = FakeGame::new();
    fake.offer::<WorldRendered>();
    fake.offer_area(Area::RENDER_WORLD_CANVAS.at(3));
    fake.offer_area(Area::RENDER_BLOCK_MESHER);
    let empty_slot = grouping(&fake, [16, 0, 16], 1);

    fake.raise_world_rendered(&FakeCanvas::default());

    let empty_slot = empty_slot.lock().unwrap();
    assert_eq!(empty_slot[0].begun, Err(ApiError::Loader(AmmlStatus::INVALID_ARGUMENT)));
}

#[test]
fn groups_need_the_area() {
    let fake = FakeGame::new();
    fake.offer::<WorldRendered>();
    fake.offer_area(Area::RENDER_WORLD_CANVAS.at(3));
    let results = grouping(&fake, [16, 16, 16], 1);

    fake.raise_world_rendered(&FakeCanvas::default());

    // The mesher is lent with the table; the loader answers the group call as it answers a build without the area.
    assert_eq!(results.lock().unwrap()[0].begun, Err(ApiError::NotAvailable));
}

#[test]
fn a_mod_holds_at_most_64_groups_in_the_fake() {
    let fake = FakeGame::new();
    fake.offer::<WorldRendered>();
    fake.offer_area(Area::RENDER_WORLD_CANVAS.at(3));
    fake.offer_area(Area::RENDER_BLOCK_MESHER);
    let seen: Arc<Mutex<Vec<Result<(), ApiError>>>> = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let listener = fake
        .ctx()
        .events()
        .listen::<WorldRendered>(HookOrder::NORMAL, move |event, _| {
            let mesher = event.mesher().unwrap();
            let groups: Vec<_> = (0..65).map(|_| mesher.begin_group([16, 16, 16])).collect();
            record
                .lock()
                .unwrap()
                .extend(groups.iter().map(|group| group.as_ref().map(|_| ()).map_err(|error| *error)));
        })
        .unwrap();
    std::mem::forget(listener);

    fake.raise_world_rendered(&FakeCanvas::default());

    let seen = seen.lock().unwrap();
    assert!(seen[..64].iter().all(Result::is_ok));
    assert_eq!(seen[64], Err(ApiError::Refused));
}

#[test]
fn a_dropped_group_gives_its_slot_back() {
    let fake = FakeGame::new();
    fake.offer::<WorldRendered>();
    fake.offer_area(Area::RENDER_WORLD_CANVAS.at(3));
    fake.offer_area(Area::RENDER_BLOCK_MESHER);
    let seen: Arc<Mutex<Vec<Result<(), ApiError>>>> = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let listener = fake
        .ctx()
        .events()
        .listen::<WorldRendered>(HookOrder::NORMAL, move |event, _| {
            let mesher = event.mesher().unwrap();
            // 65 groups begun and dropped without a finish, one at a time.
            for _ in 0..65 {
                let group = mesher.begin_group([16, 16, 16]);
                record.lock().unwrap().push(group.as_ref().map(|_| ()).map_err(|error| *error));
            }
        })
        .unwrap();
    std::mem::forget(listener);
    let canvas = FakeCanvas::default();

    fake.raise_world_rendered(&canvas);

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 65);
    assert!(seen.iter().all(Result::is_ok), "{seen:?}");
    assert_eq!(fake.open_groups(), 0);
    let discards = canvas.calls.borrow().iter().filter(|line| *line == "mesher group discard").count();
    assert_eq!(discards, 65);
}
