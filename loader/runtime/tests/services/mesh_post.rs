use amml_runtime::MeshPost;

// The top face of the unit cell at (0, 0, 0), at y = 1, in the corner order `order`.
fn top_face(order: [usize; 4]) -> Vec<[f32; 3]> {
    let corners = [[0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0]];
    order.iter().map(|corner| corners[*corner]).collect()
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-6
}

#[test]
fn faces_are_pushed_in_by_0_01() {
    // Both windings move the face into its own cell: the push does not depend on the game's winding.
    for order in [[0, 1, 2, 3], [3, 2, 1, 0]] {
        let mut positions = top_face(order);

        MeshPost::push_in(&mut positions, [0.5, 0.5, 0.5]);

        assert!(positions.iter().all(|at| close(at[1], 0.99)), "{positions:?}");
        assert!(positions.iter().all(|at| at[0] == 0.0 || at[0] == 1.0));
    }
}

#[test]
fn a_quad_through_the_cell_centre_and_a_part_quad_keep_their_place() {
    // A diagonal plane through the centre (a crossed plant) has no inside to move to.
    let mut diagonal = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 0.0]];
    let before = diagonal.clone();
    MeshPost::push_in(&mut diagonal, [0.5, 0.5, 0.5]);
    assert_eq!(diagonal, before);

    let mut three = top_face([0, 1, 2, 3])[..3].to_vec();
    let before = three.clone();
    MeshPost::push_in(&mut three, [0.5, 0.5, 0.5]);
    assert_eq!(three, before);
}

#[test]
fn one_quad_shapes_get_a_back_face() {
    // The shapes the exe names cross_texture (1), rows (6), vine (20), seagrass (97) and chain (123), and the three
    // shapes whose case draws with the cross pair writer (121, 122, 129), are two sided; a full block (0) is not.
    for shape in [1, 6, 20, 97, 121, 122, 123, 129] {
        assert!(MeshPost::is_two_sided(shape), "{shape}");
    }
    assert!(!MeshPost::is_two_sided(0));
    assert!(!MeshPost::is_two_sided(90));

    let positions = top_face([0, 1, 2, 3]);
    let back = MeshPost::reversed(&positions, |at| at).unwrap();
    assert_eq!(back, top_face([3, 2, 1, 0]));
    // Normals turn around; a vector that is not whole quads gets no back face.
    let normals = vec![[0.0, 1.0, 0.0, 0.5]; 4];
    assert_eq!(MeshPost::reversed(&normals, MeshPost::turn_normal).unwrap(), vec![[0.0, -1.0, 0.0, 0.5]; 4]);
    assert!(MeshPost::reversed(&positions[..3], |at| at).is_none());
}

#[test]
fn alpha_is_written_to_every_vertex() {
    let mut colors = vec![0xFF11_2233u32, 0x0044_5566, 0x8077_8899];

    MeshPost::set_alpha(&mut colors, 0x60);

    assert_eq!(colors, vec![0x6011_2233, 0x6044_5566, 0x6077_8899]);
}

#[test]
fn positions_move_to_the_grid_origin_and_the_box_fits_them() {
    let mut positions = vec![[10.0, 64.0, -3.0], [11.0, 65.5, -2.0]];

    MeshPost::shift(&mut positions, [10, 64, -4]);

    assert_eq!(positions, vec![[0.0, 0.0, 1.0], [1.0, 1.5, 2.0]]);
    assert_eq!(MeshPost::fit_aabb(&positions), [[0.0, 0.0, 1.0], [1.0, 1.5, 2.0]]);
    assert_eq!(MeshPost::fit_aabb(&[]), [[0.0; 3]; 2]);
}

#[test]
fn the_main_layer_comes_first_then_the_extra_layers_once() {
    // The chunk builder walks the extra bits 0 to 15h (both gate builds).
    let passes: Vec<i32> = MeshPost::passes(3, (1 << 3) | (1 << 1) | (1 << 21) | (1 << 22)).collect();

    assert_eq!(passes, vec![3, 1, 21]);
    assert_eq!(MeshPost::passes(0, 0).count(), 1);
}

// The six faces of the unit cell at (0, 0, 0): bottom, top, then the four sides, four vertices each, as the game
// writes a quad list (each quad's corners in order around the face).
fn cube() -> Vec<[f32; 3]> {
    let face = |fixed: usize, value: f32| -> Vec<[f32; 3]> {
        let (a, b) = match fixed {
            0 => (1, 2),
            1 => (0, 2),
            _ => (0, 1),
        };
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
            .iter()
            .map(|corner| {
                let mut at = [0.0; 3];
                at[fixed] = value;
                at[a] = corner[0];
                at[b] = corner[1];
                at
            })
            .collect()
    };
    [face(1, 0.0), face(1, 1.0), face(0, 0.0), face(0, 1.0), face(2, 0.0), face(2, 1.0)].concat()
}

#[test]
fn a_grey_block_takes_the_tint_and_keeps_its_alpha() {
    let positions = cube();
    let mut colors = vec![0x80FF_FFFFu32; positions.len()];

    assert!(MeshPost::tint(&positions, &mut colors, [0.5; 3], [0.5, 1.0, 0.25], false));

    assert!(colors.iter().all(|color| *color == 0x8040_FF80), "{colors:x?}");
}

#[test]
fn a_block_that_has_a_colour_is_left_alone() {
    let positions = cube();
    // One vertex is not grey: the tessellator did tint this block.
    let mut colors = vec![0xFF80_8080u32; positions.len()];
    colors[5] = 0xFF00_8040;
    let before = colors.clone();

    assert!(!MeshPost::tint(&positions, &mut colors, [0.5; 3], [0.5, 1.0, 0.25], false));

    assert_eq!(colors, before);
}

#[test]
fn a_black_tint_and_a_part_list_change_nothing() {
    let positions = cube();
    let mut colors = vec![0xFFFF_FFFFu32; positions.len()];
    assert!(!MeshPost::tint(&positions, &mut colors, [0.5; 3], [0.0, 0.0, 0.0], false));
    assert!(colors.iter().all(|color| *color == 0xFFFF_FFFF));

    // Colours that do not cover the positions are not the same list.
    let mut short = vec![0xFFFF_FFFFu32; positions.len() - 1];
    assert!(!MeshPost::tint(&positions, &mut short, [0.5; 3], [0.5; 3], false));
    // Channels past 1 are clamped.
    let mut bright = vec![0xFFFF_FFFFu32; positions.len()];
    assert!(MeshPost::tint(&positions, &mut bright, [0.5; 3], [2.0, 0.5, -1.0], false));
    assert!(bright.iter().all(|color| *color == 0xFF00_80FF), "{bright:x?}");
}

#[test]
fn the_top_only_rule_tints_the_up_facing_quad_only() {
    let positions = cube();
    let mut colors = vec![0xFFFF_FFFFu32; positions.len()];

    assert!(MeshPost::tint(&positions, &mut colors, [0.5; 3], [0.5, 1.0, 0.25], true));

    // Quad 1 is the top face; the winding does not matter, only that its outward normal points up.
    for (quad, quad_colors) in colors.chunks(4).enumerate() {
        let want = if quad == 1 { 0xFF40_FF80 } else { 0xFFFF_FFFF };
        assert!(quad_colors.iter().all(|color| *color == want), "quad {quad}: {quad_colors:x?}");
    }
    // The bottom face turned around is still not up.
    let mut turned = positions.clone();
    turned[4..8].reverse();
    let mut again = vec![0xFFFF_FFFFu32; turned.len()];
    assert!(MeshPost::tint(&turned, &mut again, [0.5; 3], [0.5, 1.0, 0.25], true));
    assert!(again[..4].iter().all(|color| *color == 0xFFFF_FFFF));
}
