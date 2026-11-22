use amml_runtime::{SkullFace, SkullTransform, WorldTransform};

// The constants of 70.26 `SkullBlockRenderer::render` (0x8939da0) and `renderSkullUpright` (0x893abe0), as the bytes
// the instructions load (survey section 7.6, rows K1..K7; `t1-ev-skull.txt`).
// 0x12dfdd6c, `addss` at 0x893a166 (floor y) and 0x893ac7a (upright y).
const FLOOR_Y: u32 = 0x38D1_B717;
// 0x12c77cd0, `addss` at 0x893a41f (wall y).
const WALL_Y: u32 = 0x3E80_0000;
const HALF: u32 = 0x3F00_0000;
// 0x1313fd40 / 0x1313fd50, the far offset of faces 2 and 4.
const FAR: u32 = 0x3F3D_70A4;
// 0x1313fd30 / 0x1313fd60, the near offset of faces 3 and 5.
const NEAR: u32 = 0x3E85_1EB8;
// 0x12dfdc50, `mulss` at 0x893a18e: degrees to radians.
const DEGREES_TO_RADIANS: u32 = 0x3C8E_FA35;
// 0x12c78b50 (face 3, 0x893a5ae; upright, 0x893ad02).
const PI: u32 = 0x4049_0FDB;
// 0x1308fb30 (face 4, 0x893a44f).
const THREE_HALF_PI: u32 = 0x4096_CBE4;
// 0x12c78b54 (face 5, 0x893a5d3).
const HALF_PI: u32 = 0x3FC9_0FDB;

fn exe(bits: u32) -> f32 {
    f32::from_bits(bits)
}

fn row(offset: [u32; 3], angle: f32) -> SkullFace {
    SkullFace { offset: offset.map(exe), angle }
}

// The translation column of the game's transform of a skull at the origin: the offset of its row.
fn translation(face: u8, turn: u8) -> [f32; 3] {
    let matrix = SkullTransform::game(face, turn, [0.0; 3]);
    [matrix[3][0], matrix[3][1], matrix[3][2]]
}

fn assert_row(face: u8, expected: SkullFace) {
    for turn in [0, 4] {
        let found = SkullTransform::face(face, turn);
        assert_eq!(found.offset.map(f32::to_bits), expected.offset.map(f32::to_bits), "face {face} turn {turn}");
        assert_eq!(found.angle.to_bits(), expected.angle.to_bits(), "face {face} turn {turn}");
        assert_eq!(translation(face, turn).map(f32::to_bits), expected.offset.map(f32::to_bits));
    }
}

#[test]
fn floor_face_turn_0_matches_the_exe_row() {
    let found = SkullTransform::face(1, 0);

    assert_eq!(found, row([HALF, FLOOR_Y, HALF], 0.0));
    assert_eq!(translation(1, 0).map(f32::to_bits), [HALF, FLOOR_Y, HALF]);
    // The 26.63 `renderSkull` takes the turn in degrees.
    assert_eq!(SkullTransform::degrees(1, 0), 0.0);
}

#[test]
fn floor_face_turn_4_matches_the_exe_row() {
    let found = SkullTransform::face(1, 4);

    // Row K1: the rotation in degrees (4 sixteenths: 90) times the exe's degrees-to-radians constant, in f32.
    let angle = 90.0f32 * exe(DEGREES_TO_RADIANS);
    assert_eq!(found.offset.map(f32::to_bits), [HALF, FLOOR_Y, HALF]);
    assert_eq!(found.angle.to_bits(), angle.to_bits());
    assert_eq!(SkullTransform::degrees(1, 4), 90.0);
}

#[test]
fn face_2_matches_the_exe_row() {
    assert_row(2, row([HALF, WALL_Y, FAR], 0.0));
}

#[test]
fn face_3_matches_the_exe_row() {
    assert_row(3, row([HALF, WALL_Y, NEAR], exe(PI)));
}

#[test]
fn face_4_matches_the_exe_row() {
    assert_row(4, row([FAR, WALL_Y, HALF], exe(THREE_HALF_PI)));
}

#[test]
fn face_5_matches_the_exe_row() {
    assert_row(5, row([NEAR, WALL_Y, HALF], exe(HALF_PI)));
}

#[test]
fn wall_faces_match_the_exe_rows() {
    // Face 0 takes the face 5 row (the `else` branch at 0x893a5cb), and a wall skull ignores the turn: the 26.63
    // `render` passes rotation 0 for every face but 1.
    assert_row(0, row([NEAR, WALL_Y, HALF], exe(HALF_PI)));
    for face in [0, 2, 3, 4, 5] {
        assert_eq!(SkullTransform::face(face, 4), SkullTransform::face(face, 0));
        assert_eq!(SkullTransform::degrees(face, 4), 0.0);
    }
}

#[test]
fn upright_offset_is_cancelled() {
    // Row K7: `renderSkullUpright` translates by the position plus the floor y, negates columns 0 and 1, turns by pi.
    let upright = SkullTransform::upright([0.0; 3]);
    assert_eq!([upright[3][0], upright[3][1], upright[3][2]].map(f32::to_bits), [0, FLOOR_Y, 0]);
    for pos in [[0.0, 0.0, 0.0], [3.0, -5.0, 7.5], [-12.0, 64.0, 0.25]] {
        for face in 0..=5 {
            for turn in [0, 4] {
                // The loader's transform on the stack top, then `renderSkullUpright`'s own, give the game's pose.
                let drawn =
                    WorldTransform::compose(&SkullTransform::matrix(face, turn, pos), &SkullTransform::upright(pos));
                let game = SkullTransform::game(face, turn, pos);
                for column in 0..4 {
                    for line in 0..4 {
                        let (found, wanted) = (drawn[column][line], game[column][line]);
                        assert!(
                            (found - wanted).abs() <= 1e-4,
                            "face {face} turn {turn} pos {pos:?} [{column}][{line}]: {found} != {wanted}"
                        );
                    }
                }
            }
        }
    }
}
