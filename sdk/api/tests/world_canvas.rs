use std::sync::{Arc, Mutex};

use amml_abi::{AmmlStatus, WorldRendered};
use amml_api::testing::{FakeCanvas, FakeGame};
use amml_api::{
    ApiError, Area, Blend, BlockPos, ColorQuad, HookOrder, MeshId, MeshLight, MeshStyle, MeshTexture, Rgba, Transform,
    Vec3, WorldCanvas, WorldRenderedExt,
};

fn camera() -> Vec3 {
    Vec3 { x: 12.5, y: 70.0, z: -3.25 }
}

#[test]
fn camera_position_comes_from_the_fake() {
    let fake = FakeGame::new();
    fake.offer::<WorldRendered>();
    fake.offer_area(Area::RENDER_WORLD_CANVAS);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<WorldRendered>(HookOrder::NORMAL, move |event, _| {
            let position = event.canvas().and_then(|canvas| canvas.camera_position());
            record.lock().unwrap().push((event.frame_id(), event.vibrant(), position));
        })
        .unwrap();

    let first =
        fake.raise_world_rendered(&FakeCanvas { camera_position: Some(camera()), vibrant: true, ..Default::default() });
    // A frame the game has no view in gives no position; the frame id is new for each dispatch.
    fake.raise_world_rendered(&FakeCanvas::default());

    assert!(!first.panicked);
    assert_eq!(*seen.lock().unwrap(), vec![(1, true, Some(camera())), (2, false, None)]);
}

#[test]
fn canvas_is_none_for_a_copy_of_the_event() {
    let fake = FakeGame::new();
    fake.offer::<WorldRendered>();
    fake.offer_area(Area::RENDER_WORLD_CANVAS);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let kept = Arc::new(Mutex::new(None));
    let keep = kept.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<WorldRendered>(HookOrder::NORMAL, move |event, _| {
            let copy = *event;
            record.lock().unwrap().push((event.canvas().is_some(), copy.canvas().is_some()));
            *keep.lock().unwrap() = Some(copy);
        })
        .unwrap();

    fake.raise_world_rendered(&FakeCanvas { camera_position: Some(camera()), ..Default::default() });

    assert_eq!(*seen.lock().unwrap(), vec![(true, false)]);
    // An event kept past its call gets no canvas either.
    assert!(kept.lock().unwrap().unwrap().canvas().is_none());
}

#[test]
fn canvas_is_none_while_the_area_is_not_offered() {
    let fake = FakeGame::new();
    fake.offer::<WorldRendered>();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<WorldRendered>(HookOrder::NORMAL, move |event, _| {
            let position = event.canvas().and_then(|canvas| canvas.camera_position());
            record.lock().unwrap().push(position);
        })
        .unwrap();

    fake.raise_world_rendered(&FakeCanvas { camera_position: Some(camera()), ..Default::default() });

    assert_eq!(*seen.lock().unwrap(), vec![None]);
}

#[test]
fn a_loader_without_a_render_table_gives_no_canvas() {
    let fake = FakeGame::with_v9_services();
    fake.offer::<WorldRendered>();
    fake.offer_area(Area::RENDER_WORLD_CANVAS);
    assert!(!fake.ctx().supports(Area::RENDER_WORLD_CANVAS));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<WorldRendered>(HookOrder::NORMAL, move |event, _| {
            record.lock().unwrap().push(event.canvas().is_some());
        })
        .unwrap();

    fake.raise_world_rendered(&FakeCanvas::default());

    assert_eq!(*seen.lock().unwrap(), vec![false]);
}

#[test]
fn free_goes_through_the_loader_and_a_stale_id_is_refused() {
    let fake = FakeGame::new();
    fake.offer::<WorldRendered>();
    fake.offer_area(Area::RENDER_WORLD_CANVAS);
    let live = fake.add_mesh();
    let left = fake.add_mesh();
    fake.leave_mesh_world();
    let after_leave = fake.add_mesh();
    let results = Arc::new(Mutex::new(Vec::new()));
    let record = results.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<WorldRendered>(HookOrder::NORMAL, move |event, _| {
            let canvas = event.canvas().unwrap();
            let mut log = record.lock().unwrap();
            log.push(canvas.free(after_leave));
            // A second free of the same id and an id of an older generation are both stale.
            log.push(canvas.free(after_leave));
            log.push(canvas.free(left));
            log.push(canvas.free(MeshId { id: 9999, generation: after_leave.generation }));
        })
        .unwrap();
    let canvas = FakeCanvas::default();

    fake.raise_world_rendered(&canvas);

    assert_eq!(
        *results.lock().unwrap(),
        vec![Ok(()), Err(ApiError::Stale), Err(ApiError::Stale), Err(ApiError::Stale)]
    );
    assert_eq!(*canvas.frees.borrow(), vec![after_leave]);
    assert_eq!(fake.live_meshes(), 0);
    let _ = live;
}

fn quad(x: f32) -> ColorQuad {
    let corner = |dx: f32, dz: f32| Vec3 { x: x + dx, y: 0.0, z: dz };
    ColorQuad {
        corners: [corner(0.0, 0.0), corner(1.0, 0.0), corner(1.0, 1.0), corner(0.0, 1.0)],
        rgba: Rgba::default(),
    }
}

#[test]
fn build_records_quads_and_style() {
    let fake = FakeGame::new();
    fake.offer::<WorldRendered>();
    fake.offer_area(Area::RENDER_WORLD_CANVAS.at(2));
    let results = Arc::new(Mutex::new(Vec::new()));
    let record = results.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<WorldRendered>(HookOrder::NORMAL, move |event, _| {
            let canvas = event.canvas().unwrap();
            let mut log = record.lock().unwrap();
            let two_sided = MeshStyle { two_sided: true, ..MeshStyle::OVERLAY };
            let mesh = canvas.build(&[quad(0.0), quad(2.0)], two_sided);
            log.push(mesh.map(|_| ()));
            let transform = Transform {
                origin: BlockPos { x: 10, y: 64, z: -3 },
                offset: Vec3 { x: 0.5, y: 0.0, z: 0.0 },
                scale: 1.0,
                yaw_degrees: 90.0,
            };
            log.push(mesh.and_then(|mesh| canvas.draw(mesh, transform)));
            // The limits of this version.
            log.push(canvas.build(&vec![quad(0.0); WorldCanvas::MAX_QUADS + 1], MeshStyle::OVERLAY).map(|_| ()));
            log.push(
                canvas.build(&[quad(0.0)], MeshStyle { light: MeshLight::World, ..MeshStyle::OVERLAY }).map(|_| ()),
            );
            log.push(
                canvas
                    .build(&[quad(0.0)], MeshStyle { texture: MeshTexture::TerrainAtlas, ..MeshStyle::OVERLAY })
                    .map(|_| ()),
            );
            log.push(canvas.build(&[quad(0.0)], MeshStyle { blend: Blend::Opaque, ..MeshStyle::OVERLAY }).map(|_| ()));
            log.push(canvas.build(&[], MeshStyle::OVERLAY).map(|_| ()));
        })
        .unwrap();
    let canvas = FakeCanvas::default();

    fake.raise_world_rendered(&canvas);

    assert_eq!(
        *results.lock().unwrap(),
        vec![
            Ok(()),
            Ok(()),
            Err(ApiError::Refused),
            Err(ApiError::NotAvailable),
            Err(ApiError::NotAvailable),
            Err(ApiError::NotAvailable),
            Err(ApiError::Loader(AmmlStatus::INVALID_ARGUMENT)),
        ]
    );
    assert_eq!(*canvas.calls.borrow(), vec!["build quads=2 style=alpha/1/0/2/full/white", "draw 1 10.5 64 -3 yaw=90"]);
    assert_eq!(fake.live_meshes(), 1);
}

#[test]
fn draw_after_a_graphics_switch_is_stale() {
    let fake = FakeGame::new();
    fake.offer::<WorldRendered>();
    fake.offer_area(Area::RENDER_WORLD_CANVAS.at(2));
    let built = Arc::new(Mutex::new(None));
    let results = Arc::new(Mutex::new(Vec::new()));
    let (keep, record) = (built.clone(), results.clone());
    let _listener = fake
        .ctx()
        .events()
        .listen::<WorldRendered>(HookOrder::NORMAL, move |event, _| {
            let canvas = event.canvas().unwrap();
            let mut kept = keep.lock().unwrap();
            let mesh = *kept.get_or_insert_with(|| canvas.build(&[quad(0.0)], MeshStyle::OVERLAY).unwrap());
            let transform =
                Transform { origin: BlockPos::default(), offset: Vec3::default(), scale: 1.0, yaw_degrees: 0.0 };
            record.lock().unwrap().push(canvas.draw(mesh, transform));
        })
        .unwrap();

    fake.raise_world_rendered(&FakeCanvas::default());
    fake.switch_graphics();
    fake.raise_world_rendered(&FakeCanvas::default());

    assert_eq!(*results.lock().unwrap(), vec![Ok(()), Err(ApiError::Stale)]);
    assert_eq!(fake.live_meshes(), 0);
}

#[test]
fn build_needs_the_version_2_canvas() {
    let fake = FakeGame::new();
    fake.offer::<WorldRendered>();
    fake.offer_area(Area::RENDER_WORLD_CANVAS);
    let results = Arc::new(Mutex::new(Vec::new()));
    let record = results.clone();
    let _listener = fake
        .ctx()
        .events()
        .listen::<WorldRendered>(HookOrder::NORMAL, move |event, _| {
            let canvas = event.canvas().unwrap();
            record.lock().unwrap().push(canvas.build(&[quad(0.0)], MeshStyle::OVERLAY).map(|_| ()));
        })
        .unwrap();

    fake.raise_world_rendered(&FakeCanvas::default());

    assert_eq!(*results.lock().unwrap(), vec![Err(ApiError::NotAvailable)]);
}
