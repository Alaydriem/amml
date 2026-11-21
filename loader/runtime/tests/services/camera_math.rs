use amml_abi::AmmlRotation;
use amml_runtime::CameraMath;

fn close(a: [f32; 3], b: [f32; 3]) -> bool {
    (0..3).all(|axis| (a[axis] - b[axis]).abs() < 1e-5)
}

fn rotation(yaw: f32, pitch: f32) -> AmmlRotation {
    AmmlRotation { yaw, pitch }
}

#[test]
fn yaw_zero_looks_along_positive_z() {
    let basis = CameraMath::basis(rotation(0.0, 0.0), 0.0);

    assert!(close(basis.forward, [0.0, 0.0, 1.0]), "{:?}", basis.forward);
    assert!(close(basis.right, [-1.0, 0.0, 0.0]), "{:?}", basis.right);
    assert!(close(basis.up, [0.0, 1.0, 0.0]), "{:?}", basis.up);
}

#[test]
fn yaw_90_looks_along_minus_x_and_pitch_down_looks_down() {
    assert!(close(CameraMath::basis(rotation(90.0, 0.0), 0.0).forward, [-1.0, 0.0, 0.0]));
    assert!(close(CameraMath::basis(rotation(0.0, 90.0), 0.0).forward, [0.0, -1.0, 0.0]));
}

#[test]
fn straight_down_has_a_finite_basis() {
    let basis = CameraMath::basis(rotation(45.0, 90.0), 0.0);

    let all = [basis.forward, basis.right, basis.up].concat();
    assert!(all.iter().all(|value| value.is_finite()), "{all:?}");
    assert!(close(basis.right, CameraMath::basis(rotation(45.0, 0.0), 0.0).right));
}

#[test]
fn roll_90_turns_the_camera_up_to_the_old_right() {
    let level = CameraMath::basis(rotation(20.0, -15.0), 0.0);

    let rolled = CameraMath::basis(rotation(20.0, -15.0), 90.0);

    assert!(close(rolled.forward, level.forward));
    assert!(close(rolled.up, level.right), "{:?} {:?}", rolled.up, level.right);
    assert!(close(rolled.right, level.up.map(|value| -value)));
}

#[test]
fn the_view_matrix_is_look_at_from_the_origin() {
    let basis = CameraMath::basis(rotation(90.0, 10.0), 5.0);

    for axis in 0..3 {
        assert!((basis.view[axis][0] - basis.right[axis]).abs() < 1e-6);
        assert!((basis.view[axis][1] - basis.up[axis]).abs() < 1e-6);
        assert!((basis.view[axis][2] + basis.forward[axis]).abs() < 1e-6);
        assert_eq!(basis.view[3][axis], 0.0);
    }
    assert_eq!(basis.view[3][3], 1.0);
}

#[test]
fn the_inverse_undoes_the_view() {
    let basis = CameraMath::basis(rotation(37.0, -20.0), 12.0);

    for row in 0..4 {
        for column in 0..4 {
            let product: f32 = (0..4).map(|k| basis.view[k][row] * basis.inverse[column][k]).sum();
            let expected = if row == column { 1.0 } else { 0.0 };
            assert!((product - expected).abs() < 1e-5, "{row} {column} {product}");
        }
    }
}

#[test]
fn projection_gives_the_vertical_fov_and_the_aspect() {
    // A perspective matrix with fov y 70 degrees and aspect 16 / 9.
    let focal_y = 1.0 / (35.0f32).to_radians().tan();
    let focal_x = focal_y / (16.0 / 9.0);
    let mut matrix = [[0.0f32; 4]; 4];
    matrix[0][0] = focal_x;
    matrix[1][1] = focal_y;

    let (fov_y, aspect) = CameraMath::projection(&matrix).unwrap();

    assert!((fov_y - 70.0).abs() < 1e-3, "{fov_y}");
    assert!((aspect - 16.0 / 9.0).abs() < 1e-5, "{aspect}");
}

#[test]
fn projection_of_a_matrix_without_focal_lengths_is_none() {
    assert_eq!(CameraMath::projection(&[[0.0; 4]; 4]), None);
    let mut matrix = [[0.0f32; 4]; 4];
    matrix[0][0] = f32::NAN;
    matrix[1][1] = 1.0;
    assert_eq!(CameraMath::projection(&matrix), None);
}
