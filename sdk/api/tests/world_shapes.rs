use amml_api::testing::FakeGame;
use amml_api::{ApiError, ApplyStats, Area, Rgba, ShapeOp, Vec3};

const RED: Rgba = Rgba { r: 255, g: 0, b: 51, a: 255 };

fn from() -> Vec3 {
    Vec3 { x: 1.0, y: 2.0, z: 3.0 }
}

fn to() -> Vec3 {
    Vec3 { x: 4.0, y: 5.0, z: 6.0 }
}

fn offered() -> FakeGame {
    let fake = FakeGame::new();
    fake.offer_area(Area::WORLD_SHAPES);
    fake.open_gate();
    fake.set_epoch(1);
    fake
}

#[test]
fn apply_records_ops() {
    let fake = offered();
    fake.set_target_epoch(3);
    let shapes = fake.ctx().world_shapes();
    let ops = [
        ShapeOp::Line { id: 1, from: from(), to: to(), color: RED },
        ShapeOp::Box { id: 2, from: from(), to: to(), color: RED },
        ShapeOp::Arrow { id: 3, from: from(), to: to(), color: RED },
        ShapeOp::Remove { id: 2 },
    ];

    let stats = shapes.apply(-1, &ops).unwrap();

    assert_eq!(stats.delivered, 4);
    assert_eq!(
        fake.shape_ops(),
        [
            "line id=1 from=1,2,3 to=4,5,6 color=255,0,51,255 dim=-1",
            "box id=2 from=1,2,3 to=4,5,6 color=255,0,51,255 dim=-1",
            "arrow id=3 from=1,2,3 to=4,5,6 color=255,0,51,255 dim=-1",
            "remove id=2 dim=-1",
        ]
    );
}

#[test]
fn target_epoch_comes_from_the_fake() {
    let fake = offered();
    let shapes = fake.ctx().world_shapes();
    assert_eq!(shapes.target_epoch(), None);

    fake.set_target_epoch(7);
    assert_eq!(shapes.target_epoch(), Some(7));

    // A new epoch means the game dropped the shapes, so the mod sends them again.
    fake.set_target_epoch(8);
    assert_eq!(shapes.target_epoch(), Some(8));
}

#[test]
fn apply_before_capture_is_not_ready() {
    let fake = offered();
    let shapes = fake.ctx().world_shapes();

    assert_eq!(shapes.apply(0, &[ShapeOp::Remove { id: 1 }]), Err(ApiError::NotReady));
    assert!(fake.shape_ops().is_empty());
}

#[test]
fn a_batch_over_the_maximum_is_refused() {
    let fake = offered();
    fake.set_target_epoch(1);
    let shapes = fake.ctx().world_shapes();
    assert_eq!(shapes.max_batch(), 800);
    let ops: Vec<ShapeOp> = (0..801).map(|id| ShapeOp::Line { id, from: from(), to: to(), color: RED }).collect();

    assert_eq!(shapes.apply(0, &ops), Err(ApiError::Refused));
    assert!(fake.shape_ops().is_empty());
    assert_eq!(shapes.apply(0, &ops[..800]), Ok(ApplyStats { delivered: 800, elapsed_us: 0 }));
}

#[test]
fn without_the_area_the_shapes_are_not_available() {
    let fake = FakeGame::new();
    fake.open_gate();
    fake.set_epoch(1);
    fake.set_target_epoch(1);
    let shapes = fake.ctx().world_shapes();

    assert_eq!(shapes.apply(0, &[ShapeOp::Remove { id: 1 }]), Err(ApiError::NotAvailable));
    assert_eq!(shapes.target_epoch(), None);
}

#[test]
fn delivered_counts_payloads_as_the_loader_does() {
    let fake = offered();
    fake.set_target_epoch(1);
    let shapes = fake.ctx().world_shapes();
    shapes.apply(0, &[ShapeOp::Line { id: 1, from: from(), to: to(), color: RED }]).unwrap();

    // A kind change of a live id is two payloads (remove, new draw); a remove of an unknown id is none.
    let ops = [ShapeOp::Arrow { id: 1, from: from(), to: to(), color: RED }, ShapeOp::Remove { id: 9 }];
    assert_eq!(shapes.apply(0, &ops).unwrap().delivered, 2);
    assert_eq!(shapes.apply(0, &[ShapeOp::Remove { id: 1 }]).unwrap().delivered, 1);

    // A new target epoch: the game dropped the shapes, so a draw of id 1 is a new shape again.
    shapes.apply(0, &[ShapeOp::Box { id: 1, from: from(), to: to(), color: RED }]).unwrap();
    fake.set_target_epoch(2);
    let again = shapes.apply(0, &[ShapeOp::Line { id: 1, from: from(), to: to(), color: RED }]).unwrap();
    assert_eq!(again.delivered, 1);
}
