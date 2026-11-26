use amml_runtime::{CacheWindow, WindowCell};

// The grid index of (x, y, z) in a grid of `dims`: x first, then z, then y.
fn grid_index(dims: [u32; 3], x: u32, y: u32, z: u32) -> usize {
    ((y * dims[2] + z) * dims[0] + x) as usize
}

#[test]
fn grid_cells_land_at_the_exe_index_order() {
    let dims = [3, 4, 5];
    let mut cells = vec![CacheWindow::EMPTY; 60];
    cells[grid_index(dims, 2, 1, 3)] = 7;
    cells[grid_index(dims, 0, 3, 4)] = 9;

    let window = CacheWindow::place(dims, &cells).unwrap();

    // The window index is x * 400 + y * 20 + z on both gate builds (survey section 7, row M4).
    assert_eq!(window.len(), 8000);
    assert_eq!(window[2 * 400 + 20 + 3], WindowCell::Palette(7));
    assert_eq!(window[3 * 20 + 4], WindowCell::Palette(9));
    assert_eq!(CacheWindow::index(2, 1, 3), 823);
    assert_eq!(window.iter().filter(|cell| matches!(cell, WindowCell::Palette(_))).count(), 2);
}

#[test]
fn border_cells_are_air() {
    let dims = [2, 2, 2];
    let mut cells = vec![1u16; 8];
    cells[grid_index(dims, 1, 1, 1)] = CacheWindow::EMPTY;

    let window = CacheWindow::place(dims, &cells).unwrap();

    // An empty grid cell and every window cell outside the grid are air, so ghost cells join each other, not the world.
    assert_eq!(window[CacheWindow::index(1, 1, 1)], WindowCell::Air);
    assert_eq!(window[CacheWindow::index(0, 0, 0)], WindowCell::Palette(1));
    assert_eq!(window[CacheWindow::index(2, 0, 0)], WindowCell::Air);
    assert_eq!(window[CacheWindow::index(0, 2, 0)], WindowCell::Air);
    assert_eq!(window[CacheWindow::index(19, 19, 19)], WindowCell::Air);
    assert_eq!(window.iter().filter(|cell| **cell != WindowCell::Air).count(), 7);
}

#[test]
fn a_grid_that_does_not_fit_is_not_placed() {
    assert!(CacheWindow::place([21, 1, 1], &[0; 21]).is_none());
    assert!(CacheWindow::place([2, 2, 2], &[0; 7]).is_none());
    assert!(CacheWindow::place([20, 20, 20], &vec![0; 8000]).is_some());
}

#[test]
fn only_cells_inside_the_one_cell_border_are_drawn() {
    let inner: Vec<[u32; 3]> = CacheWindow::interior([3, 4, 3]).collect();

    assert_eq!(inner, vec![[1, 1, 1], [1, 2, 1]]);
    assert_eq!(CacheWindow::interior([2, 5, 5]).count(), 0);
    assert_eq!(CacheWindow::grid_index([3, 4, 3], 1, 2, 1), grid_index([3, 4, 3], 1, 2, 1));
}
