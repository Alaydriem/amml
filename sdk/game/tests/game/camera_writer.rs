use std::cell::Cell;
use std::ffi::c_void;

use amml_game::{CameraWriter, EntryId, GameEntries};

use crate::support::FakeEntries;

const VIEW_STACK: u64 = 0;
const DIRTY: u64 = 0x38;
const INVERSE: u64 = 0xC0;
const RIGHT: u64 = 0x100;
const UP: u64 = 0x10C;
const FORWARD: u64 = 0x118;

thread_local! {
    // the number of calls, and whether every write was in the camera when the last call came
    static UPDATE: Cell<(u32, bool)> = const { Cell::new((0, false)) };
}

/// A camera of 0x200 bytes.
#[repr(C, align(16))]
struct FakeCamera {
    bytes: [u8; 0x200],
}

/// A fake camera whose view stack, at `VIEW_STACK`, is a deque of two one-matrix blocks with the top in the second.
struct Stack {
    camera: Box<FakeCamera>,
    blocks: Box<[[f32; 16]; 2]>,
    _map: Box<[*mut [f32; 16]; 2]>,
}

impl Stack {
    fn new() -> Self {
        let mut blocks = Box::new([[1.0f32; 16], [2.0f32; 16]]);
        let map = Box::new([(&raw mut blocks[0]), (&raw mut blocks[1])]);
        let mut camera = Box::new(FakeCamera { bytes: [0; 0x200] });
        let words = [0usize, map.as_ptr() as usize, 2, 1, 1];
        for (index, word) in words.iter().enumerate() {
            let at = VIEW_STACK as usize + index * 8;
            camera.bytes[at..at + 8].copy_from_slice(&word.to_le_bytes());
        }
        Self { camera, blocks, _map: map }
    }

    fn ptr(&mut self) -> *mut c_void {
        (&raw mut *self.camera).cast()
    }

    fn floats(&self, at: u64, count: usize) -> Vec<f32> {
        (0..count)
            .map(|i| {
                let at = at as usize + i * 4;
                f32::from_le_bytes(self.camera.bytes[at..at + 4].try_into().unwrap())
            })
            .collect()
    }
}

extern "C" fn update(camera: *mut c_void) {
    // SAFETY: the writer passes the fake camera.
    let bytes = unsafe { &(*camera.cast::<FakeCamera>()).bytes };
    let float = |at: u64| f32::from_le_bytes(bytes[at as usize..at as usize + 4].try_into().unwrap());
    let written = bytes[DIRTY as usize] == 1
        && float(INVERSE) == 5.0
        && float(RIGHT) == 1.0
        && float(UP + 4) == 1.0
        && float(FORWARD + 8) == 1.0;
    UPDATE.with(|state| {
        let (calls, _) = state.get();
        state.set((calls + 1, written));
    });
}

fn values() -> Vec<(EntryId, u64)> {
    vec![
        (EntryId::CAMERA_FORWARD, FORWARD),
        (EntryId::CAMERA_UPDATE_VIEW_MATRIX_DEPENDENCIES, (update as extern "C" fn(*mut c_void)) as usize as u64),
        (EntryId::LEVEL_RENDERER_CAMERA_PRE_RENDER_UPDATE, 0x1000),
        (EntryId::LEVEL_RENDERER_CAMERA_CAMERA_POS, 0x20),
        (EntryId::LEVEL_RENDERER_CAMERA_CAMERA_TARGET_POS, 0x2C),
        (EntryId::VIEW_RENDER_OBJECT_VIEW_DATA, 0x40),
        (EntryId::VIEW_RENDER_DATA_CAMERA_POS, 0),
    ]
}

fn entries() -> GameEntries {
    FakeEntries::build(&values())
}

#[test]
fn the_feature_holds_every_write_entry() {
    let ids: Vec<EntryId> = values().into_iter().map(|(id, _)| id).collect();
    for id in CameraWriter::ENTRIES {
        assert!(ids.contains(&id), "{} has no test value", id.as_str());
    }
    assert_eq!(CameraWriter::ENTRIES.len(), ids.len());
}

#[test]
fn new_is_none_when_any_write_entry_is_missing() {
    assert!(CameraWriter::new(&entries()).is_some());
    for (missing, _) in values() {
        let without: Vec<_> = values().into_iter().filter(|(id, _)| *id != missing).collect();
        assert!(CameraWriter::new(&FakeEntries::build(&without)).is_none(), "{} missing", missing.as_str());
    }
}

#[test]
fn new_is_none_when_the_forward_anchor_is_not_where_the_layout_has_it() {
    let moved: Vec<_> = values()
        .into_iter()
        .map(|(id, value)| if id == EntryId::CAMERA_FORWARD { (id, FORWARD + 0x10) } else { (id, value) })
        .collect();

    assert!(CameraWriter::new(&FakeEntries::build(&moved)).is_none());
}

#[test]
fn write_rotation_writes_the_stack_top_and_the_basis_then_calls_the_update_once() {
    let writer = CameraWriter::new(&entries()).unwrap();
    let mut stack = Stack::new();
    let view: [[f32; 4]; 4] = std::array::from_fn(|c| std::array::from_fn(|r| (c * 4 + r) as f32));
    let inverse = [[5.0; 4]; 4];

    // SAFETY: a live fake camera with the layout the entries name.
    unsafe { writer.write_rotation(stack.ptr(), [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], view, inverse) };

    assert_eq!(stack.blocks[1].to_vec(), (0..16).map(|i| i as f32).collect::<Vec<_>>());
    assert_eq!(stack.blocks[0], [1.0; 16]);
    assert_eq!(stack.camera.bytes[DIRTY as usize], 1);
    assert_eq!(stack.floats(INVERSE, 16), vec![5.0; 16]);
    assert_eq!(stack.floats(RIGHT, 3), vec![1.0, 0.0, 0.0]);
    assert_eq!(stack.floats(UP, 3), vec![0.0, 1.0, 0.0]);
    assert_eq!(stack.floats(FORWARD, 3), vec![0.0, 0.0, 1.0]);
    assert_eq!(UPDATE.with(Cell::get), (1, true));
}

#[test]
fn write_rotation_on_an_empty_stack_writes_nothing() {
    let writer = CameraWriter::new(&entries()).unwrap();
    let mut stack = Stack::new();
    let size_at = VIEW_STACK as usize + 32;
    stack.camera.bytes[size_at..size_at + 8].copy_from_slice(&0usize.to_le_bytes());

    // SAFETY: a live fake camera with the layout the entries name.
    unsafe { writer.write_rotation(stack.ptr(), [0.0; 3], [0.0; 3], [0.0; 3], [[9.0; 4]; 4], [[9.0; 4]; 4]) };

    assert_eq!(stack.blocks[1], [2.0; 16]);
    assert_eq!(stack.camera.bytes[DIRTY as usize], 0);
    assert_eq!(UPDATE.with(Cell::get).0, 0);
}

#[test]
fn positions_go_to_both_members_of_each_object() {
    let writer = CameraWriter::new(&entries()).unwrap();
    let mut level_camera = [0.0f32; 32];
    let mut view_object = [0.0f32; 64];

    // SAFETY: live buffers large enough for the offsets the entries name.
    unsafe {
        writer.write_level_camera_position((&raw mut level_camera).cast(), [1.0, 2.0, 3.0]);
        writer.write_view_position((&raw mut view_object).cast(), [4.0, 5.0, 6.0]);
    }

    assert_eq!(level_camera[8..11], [1.0, 2.0, 3.0]);
    assert_eq!(level_camera[11..14], [1.0, 2.0, 3.0]);
    assert_eq!(view_object[16..19], [4.0, 5.0, 6.0]);
    assert_eq!(view_object[19..22], [4.0, 5.0, 6.0]);
    assert_eq!(view_object[22], 0.0);
}
