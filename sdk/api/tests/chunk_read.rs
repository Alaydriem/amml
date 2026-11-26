use std::sync::{Arc, Mutex};

use amml_abi::ClientTick;
use amml_api::testing::{FakeChunk, FakeClient, FakeGame, FakeWorld};
use amml_api::{
    Area, BlockPos, BlockSource, BlockTypeId, ChunkLoad, ColumnReadStats, ColumnRequest, ColumnSample, HookOrder, Rgba,
    Screen,
};

fn tick() -> ClientTick {
    ClientTick { header: FakeGame::header::<ClientTick>() }
}

fn in_world(world: FakeWorld) -> FakeClient {
    FakeClient {
        local_player: true,
        world_active: true,
        top_screen: Screen::Other,
        world: Some(world),
        ..FakeClient::default()
    }
}

const AREAS: [Area; 3] = [Area::CLIENT_READ, Area::ACTOR_READ.at(3), Area::WORLD_CHUNK_READ];

// Runs `body` with the block source of the local player of one `ClientTick`, after offering `areas`.
fn with_source<R: Send + 'static>(
    fake: &FakeGame,
    areas: &[Area],
    body: impl Fn(&BlockSource<'_>) -> R + Send + 'static,
) -> R {
    fake.offer::<ClientTick>();
    for area in areas {
        fake.offer_area(*area);
    }
    fake.open_gate();
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            let client = at.client().unwrap();
            let source = client.local_player().unwrap().block_source().unwrap();
            *record.lock().unwrap() = Some(body(&source));
        })
        .unwrap();
    fake.raise_with_client(&mut tick());
    let taken = seen.lock().unwrap().take();
    taken.unwrap()
}

fn stone() -> ColumnSample {
    ColumnSample { shown: BlockTypeId(7), shown_y: 62, height: 62, solid: 62, ..ColumnSample::default() }
}

fn world_of(load: ChunkLoad) -> FakeWorld {
    let mut chunk = FakeChunk::new(load);
    chunk.samples = [stone(); 256];
    let mut world = FakeWorld::default();
    world.chunks.insert((0, 0), chunk);
    world
}

#[test]
fn not_loaded_chunk_reads_not_loaded() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world_of(ChunkLoad::NotLoaded)));

    let states =
        with_source(&fake, &AREAS, |source| (source.chunk(0, 0).unwrap().load_state(), source.chunk(3, 3).is_none()));

    assert_eq!(states, (Some(ChunkLoad::NotLoaded), true));
}

#[test]
fn load_states_come_from_the_fake() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world_of(ChunkLoad::TopPlaceholder)));

    let state = with_source(&fake, &AREAS, |source| source.chunk(0, 0).unwrap().load_state());

    assert_eq!(state, Some(ChunkLoad::TopPlaceholder));
}

#[test]
fn masked_columns_only_are_written() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world_of(ChunkLoad::Loaded)));

    let (out, stats) = with_source(&fake, &AREAS, |source| {
        let mut out = [ColumnSample { shown_y: -1, ..ColumnSample::default() }; 256];
        let stats = source.chunk(0, 0).unwrap().read_columns(&ColumnRequest::one(3, 2, 70), &mut out);
        (out, stats)
    });

    let at = 2 * 16 + 3;
    assert_eq!(stats, Some(ColumnReadStats { read: 1, skipped: 0, retry: 0 }));
    assert_eq!(out[at], ColumnSample { flags: ColumnSample::NO_COLOR, biome: ColumnSample::NO_BIOME, ..stone() });
    assert!(out.iter().enumerate().all(|(index, sample)| index == at || sample.shown_y == -1));
}

#[test]
fn a_chunk_that_is_not_loaded_marks_the_asked_columns_skipped() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world_of(ChunkLoad::TopPlaceholder)));

    let (out, stats) = with_source(&fake, &AREAS, |source| {
        let mut out = [ColumnSample::default(); 256];
        let stats = source.chunk(0, 0).unwrap().read_columns(&ColumnRequest::all(70), &mut out);
        (out, stats)
    });

    assert_eq!(stats, Some(ColumnReadStats { read: 0, skipped: 256, retry: 0 }));
    assert!(out.iter().all(ColumnSample::skipped));
}

#[test]
fn retry_columns_are_counted() {
    let fake = FakeGame::new();
    let mut world = world_of(ChunkLoad::Loaded);
    if let Some(chunk) = world.chunks.get_mut(&(0, 0)) {
        chunk.samples[5].flags = ColumnSample::RETRY;
    }
    fake.set_client(in_world(world));

    let stats = with_source(&fake, &AREAS, |source| {
        let mut out = [ColumnSample::default(); 256];
        source.chunk(0, 0).unwrap().read_columns(&ColumnRequest::all(70), &mut out)
    });

    assert_eq!(stats, Some(ColumnReadStats { read: 256, skipped: 0, retry: 1 }));
}

#[test]
fn the_chunk_calls_need_their_area() {
    let fake = FakeGame::new();
    fake.set_client(in_world(world_of(ChunkLoad::Loaded)));

    let none = with_source(&fake, &[Area::CLIENT_READ, Area::ACTOR_READ.at(3)], |source| source.chunk(0, 0).is_none());

    assert!(none);
}

#[test]
fn a_loader_before_the_world_table_serves_no_chunk_area() {
    let fake = FakeGame::with_v8_services();
    fake.set_client(in_world(world_of(ChunkLoad::Loaded)));

    let none = with_source(&fake, &AREAS, |source| source.chunk(0, 0).is_none());

    assert!(none);
    assert!(!fake.ctx().supports(Area::WORLD_CHUNK_READ));
    assert_eq!(fake.ctx().blocks().type_name(BlockTypeId(7)), None);
}

#[test]
fn type_name_comes_from_the_name_table() {
    let fake = FakeGame::new();
    let mut world = world_of(ChunkLoad::Loaded);
    world.names.insert(BlockTypeId(7), "minecraft:stone".to_string());
    let long = format!("minecraft:{}", "x".repeat(300));
    world.names.insert(BlockTypeId(8), long.clone());
    fake.set_client(in_world(world));
    fake.offer_area(Area::WORLD_CHUNK_READ);
    fake.open_gate();
    let blocks = fake.ctx().blocks();

    assert_eq!(blocks.type_name(BlockTypeId(7)).as_deref(), Some("minecraft:stone"));
    assert_eq!(blocks.type_name(BlockTypeId(8)), Some(long));
    assert_eq!(blocks.type_name(BlockTypeId(9)), None);
    assert_eq!(blocks.type_name(BlockTypeId::NONE), None);
}

#[test]
fn a_column_request_selects_the_bit_of_its_column() {
    let corner = ColumnRequest::one(15, 15, 70);

    assert_eq!(ColumnRequest::one(3, 2, 70).columns, [1 << 35, 0, 0, 0]);
    assert_eq!(corner.columns, [0, 0, 0, 1 << 63]);
    assert!(corner.selects(15, 15) && !corner.selects(14, 15));
    assert_eq!(ColumnRequest::one(16, 0, 70).columns, [0; 4]);
    assert!(ColumnRequest::all(70).selects(0, 0) && !ColumnRequest::all(70).selects(16, 0));
}

const COLOR_AREAS: [Area; 4] =
    [Area::CLIENT_READ, Area::ACTOR_READ.at(3), Area::WORLD_CHUNK_READ, Area::WORLD_MAP_COLOR];

fn painted_world() -> FakeWorld {
    let mut world = world_of(ChunkLoad::Loaded);
    if let Some(chunk) = world.chunks.get_mut(&(0, 0)) {
        let paint = ColumnSample {
            shown_color: Rgba { r: 1, g: 2, b: 3, a: 255 },
            water_color: Rgba { r: 4, g: 5, b: 6, a: 255 },
            biome: 6,
            ..stone()
        };
        chunk.samples = [paint; 256];
    }
    world
}

fn first_sample(request: ColumnRequest) -> impl Fn(&BlockSource<'_>) -> ColumnSample {
    move |source| {
        let mut out = [ColumnSample::default(); 256];
        source.chunk(0, 0).unwrap().read_columns(&request, &mut out);
        out[0]
    }
}

#[test]
fn vanilla_colors_off_gives_no_color() {
    let fake = FakeGame::new();
    fake.set_client(in_world(painted_world()));

    let sample = with_source(&fake, &COLOR_AREAS, first_sample(ColumnRequest::all(70)));

    assert_eq!((sample.shown_color, sample.water_color), (Rgba::default(), Rgba::default()));
    assert_eq!(sample.flags, ColumnSample::NO_COLOR);
}

#[test]
fn vanilla_colors_without_the_map_color_area_give_no_color() {
    let fake = FakeGame::new();
    fake.set_client(in_world(painted_world()));

    let sample =
        with_source(&fake, &AREAS, first_sample(ColumnRequest { vanilla_colors: true, ..ColumnRequest::all(70) }));

    assert_eq!((sample.shown_color, sample.flags), (Rgba::default(), ColumnSample::NO_COLOR));
}

#[test]
fn vanilla_colors_with_the_area_give_the_colours_of_the_chunk() {
    let fake = FakeGame::new();
    fake.set_client(in_world(painted_world()));

    let sample = with_source(
        &fake,
        &COLOR_AREAS,
        first_sample(ColumnRequest { vanilla_colors: true, ..ColumnRequest::all(70) }),
    );

    assert_eq!(sample.shown_color, Rgba { r: 1, g: 2, b: 3, a: 255 });
    assert_eq!(sample.water_color, Rgba { r: 4, g: 5, b: 6, a: 255 });
    assert_eq!(sample.flags, 0);
}

#[test]
fn biome_name_from_the_fake_table() {
    let fake = FakeGame::new();
    let mut world = painted_world();
    world.biome_names.insert(6, "swamp".to_string());
    let long = "x".repeat(200);
    world.biome_names.insert(7, long.clone());
    fake.set_client(in_world(world));
    let areas = [Area::CLIENT_READ, Area::ACTOR_READ.at(3), Area::WORLD_CHUNK_READ, Area::WORLD_BIOMES];

    let sample = with_source(&fake, &areas, first_sample(ColumnRequest::all(70)));
    let biomes = fake.ctx().biomes();

    assert_eq!(sample.biome, 6);
    assert_eq!(biomes.name(6).as_deref(), Some("swamp"));
    assert_eq!(biomes.name(7), Some(long));
    assert_eq!(biomes.name(8), None);
    assert_eq!(biomes.name(ColumnSample::NO_BIOME), None);
}

#[test]
fn without_the_biomes_area_the_biome_is_no_biome_and_names_are_none() {
    let fake = FakeGame::new();
    let mut world = painted_world();
    world.biome_names.insert(6, "swamp".to_string());
    fake.set_client(in_world(world));

    let sample = with_source(&fake, &AREAS, first_sample(ColumnRequest::all(70)));

    assert_eq!(sample.biome, ColumnSample::NO_BIOME);
    assert_eq!(fake.ctx().biomes().name(6), None);
}

const AREAS_BIOME_AT: [Area; 4] =
    [Area::CLIENT_READ, Area::ACTOR_READ.at(3), Area::WORLD_CHUNK_READ, Area::WORLD_BIOMES.at(2)];

fn biome_world(load: ChunkLoad) -> FakeWorld {
    let mut world = world_of(load);
    world.biomes_at.insert(BlockPos { x: 3, y: 70, z: 4 }, 1);
    world.biomes_at.insert(BlockPos { x: 3, y: -40, z: 4 }, 2);
    world
}

#[test]
fn biome_at_gives_cave_biome_under_surface() {
    let fake = FakeGame::new();
    fake.set_client(in_world(biome_world(ChunkLoad::Loaded)));

    let (cave, surface) = with_source(&fake, &AREAS_BIOME_AT, |source| {
        (source.biome_at(BlockPos { x: 3, y: -40, z: 4 }), source.biome_at(BlockPos { x: 3, y: 70, z: 4 }))
    });

    assert_eq!((cave, surface), (Some(2), Some(1)));
}

#[test]
fn biome_at_unloaded_chunk_gives_none() {
    let fake = FakeGame::new();
    fake.set_client(in_world(biome_world(ChunkLoad::NotLoaded)));

    let (not_loaded, no_chunk) = with_source(&fake, &AREAS_BIOME_AT, |source| {
        (source.biome_at(BlockPos { x: 3, y: -40, z: 4 }), source.biome_at(BlockPos { x: 300, y: 70, z: 4 }))
    });

    assert_eq!((not_loaded, no_chunk), (None, None));
}

#[test]
fn biome_at_without_v2_gives_none() {
    let fake = FakeGame::new();
    fake.set_client(in_world(biome_world(ChunkLoad::Loaded)));
    fake.offer_area(Area::WORLD_BIOMES.at(2));
    fake.withdraw_area(Area::WORLD_BIOMES.at(2));
    fake.offer_area(Area::WORLD_BIOMES);

    let found = with_source(&fake, &AREAS_BIOME_AT[..3], |source| source.biome_at(BlockPos { x: 3, y: -40, z: 4 }));

    assert_eq!(found, None);
}
