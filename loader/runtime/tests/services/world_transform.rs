use amml_runtime::{WorldPlacement, WorldTransform};

fn close(a: [f32; 3], b: [f32; 3]) -> bool {
    a.iter().zip(b).all(|(a, b)| (a - b).abs() < 1e-5)
}

#[test]
fn far_origin_keeps_sub_block_precision() {
    let placement = WorldPlacement {
        origin: [30_000_000, 64, -30_000_000],
        offset: [0.25, 0.5, 0.75],
        scale: 1.0,
        yaw_degrees: 0.0,
    };
    let camera = [30_000_000.0, 64.0, -30_000_000.0];

    let matrix = WorldTransform::matrix(&placement, camera);

    // In f32, 30 000 000 + 0.25 is 30 000 000: the quarter block would be lost before the camera is taken off.
    assert_eq!(30_000_000.0f32 + 0.25 - 30_000_000.0, 0.0);
    assert!(close([matrix[3][0], matrix[3][1], matrix[3][2]], [0.25, 0.5, 0.75]), "{matrix:?}");
    assert!(close(WorldTransform::apply(&matrix, [1.0, 0.0, 0.0]), [1.25, 0.5, 0.75]));
}

#[test]
fn yaw_90_maps_plus_z_to_minus_x() {
    let placement = WorldPlacement { origin: [10, 0, 0], offset: [0.0; 3], scale: 2.0, yaw_degrees: 90.0 };

    let matrix = WorldTransform::matrix(&placement, [0.0; 3]);

    // Yaw 0 faces +Z and 90 faces -X (the `Rotation` signs); the scale applies before the move.
    assert!(close(WorldTransform::apply(&matrix, [0.0, 0.0, 1.0]), [8.0, 0.0, 0.0]));
    assert!(close(WorldTransform::apply(&matrix, [1.0, 0.0, 0.0]), [10.0, 0.0, 2.0]));
    assert!(close(WorldTransform::apply(&matrix, [0.0, 1.0, 0.0]), [10.0, 2.0, 0.0]));
}

#[test]
fn compose_applies_the_mesh_matrix_before_the_stack_top() {
    let placement = WorldPlacement { origin: [0, 0, 0], offset: [1.0, 0.0, 0.0], scale: 1.0, yaw_degrees: 0.0 };
    let mesh = WorldTransform::matrix(&placement, [0.0; 3]);
    // A stack top that moves everything 5 blocks up.
    let mut top = [[0.0f32; 4]; 4];
    for (index, column) in top.iter_mut().enumerate() {
        column[index] = 1.0;
    }
    top[3][1] = 5.0;

    let both = WorldTransform::compose(&top, &mesh);

    assert!(close(WorldTransform::apply(&both, [0.0, 0.0, 0.0]), [1.0, 5.0, 0.0]));
}
