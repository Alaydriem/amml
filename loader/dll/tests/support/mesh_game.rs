use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Mutex, OnceLock};

use amml_data::{EntryKind, Resolved};

use super::Harness;

// The game calls of the mesh path, one line each, in order.
static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The translation of the last matrix the stand-in pop saw, as f32 bits.
static POPPED: Mutex<[u32; 3]> = Mutex::new([0; 3]);
// The positions and the colours the tessellator held at the last `end`, and the box of its mesh data.
type EndData = (Vec<[f32; 3]>, Vec<u32>, [[f32; 3]; 2]);
static LAST_END: Mutex<Option<EndData>> = Mutex::new(None);
// The leaked objects of the stand-in game, made once.
static OBJECTS: OnceLock<MeshObjects> = OnceLock::new();

// The addresses of the leaked stand-in objects.
#[derive(Clone, Copy)]
struct MeshObjects {
    screen: usize,
    tessellator: usize,
    camera: usize,
    shader_color: usize,
    info: usize,
    material: usize,
    blend_info: usize,
    blend_material: usize,
    item_renderer: usize,
    camera_matrices: usize,
}

/// Stand-ins with the game signatures for every call of the world mesh path, and the objects they work on: a screen
/// context whose `Tessellator&` is at B0h and whose `MeshContext` is at 10h (render context, camera, constant buffers,
/// manager, shader colour), a tessellator laid out as the loader's mirror, a camera with its world matrix stack at 40h,
/// and the wireframe `MaterialPtr` -> `RenderMaterialInfo` (+48h) -> `RenderMaterial` chain. The render stand-in
/// records its arguments and destroys element 0 of the texture list, as the game does at the exit of
/// `Mesh::renderMesh` (the list overload).
pub struct MeshGame;

impl MeshGame {
    const TESSELLATOR_AT: usize = 0xB0;
    const TESSELLATING: usize = 0x2A0;
    const COUNT: usize = 0x298;
    // `mMeshData.mPositions` in the tessellator: first, last, end.
    const POSITIONS: usize = 0x8 + 0x8;
    // `mMeshData.mColors` and `mMeshData.mAABB` in the tessellator.
    const COLORS: usize = 0x8 + 0x68;
    const AABB: usize = 0x8 + 0x128;
    /// The `RenderMaterial` byte offsets the entries name: blend, source, destination, depth write.
    pub const MATERIAL_BYTES: [usize; 4] = [0x185, 0x180, 0x181, 0x193];
    /// The bytes the stand-in wireframe material has before any patch.
    pub const GAME_BYTES: [u8; 4] = [0, 2, 1, 1];
    /// The shader colour the frame has before a draw.
    pub const FRAME_COLOR: [f32; 4] = [0.25, 0.5, 0.75, 1.0];
    const CAMERA_DIRTY: usize = 0x40 + 0x38;
    /// `ItemInHandRenderer.mMatBlendBlock` in the stand-in item renderer.
    pub const BLEND_AT: usize = 0x98;
    /// The `BaseTextureData` and `ResourceLocation` words of the stand-in atlas `TexturePtr` of the level renderer.
    pub const ATLAS_WORDS: [usize; 2] = [0xA71A_5000, 0xA71A_6000];

    fn objects() -> MeshObjects {
        *OBJECTS.get_or_init(|| {
            let leak = |words: usize| Box::leak(vec![0u64; words].into_boxed_slice()).as_mut_ptr() as usize;
            let objects = MeshObjects {
                screen: leak(0x40),
                tessellator: leak(0x60),
                camera: leak(0x40),
                shader_color: leak(4),
                info: leak(0x10),
                material: leak(0x40),
                blend_info: leak(0x10),
                blend_material: leak(0x40),
                item_renderer: leak(0x40),
                camera_matrices: leak(0x40),
            };
            let render_context = leak(4);
            let constant_buffers = leak(0x40);
            // SAFETY: each object is a leaked zeroed block large enough for the offsets written.
            unsafe {
                Self::put(objects.screen + Self::TESSELLATOR_AT, objects.tessellator);
                Self::put(objects.screen + 0x10, render_context);
                Self::put(objects.screen + 0x18, objects.camera);
                Self::put(objects.screen + 0x20, constant_buffers);
                Self::put(objects.screen + 0x30, objects.shader_color);
                Self::put(objects.info + 0x48, objects.material);
                Self::put(objects.blend_info + 0x48, objects.blend_material);
            }
            objects
        })
    }

    // Safety: `at` is writable for a pointer.
    unsafe fn put(at: usize, value: usize) {
        // SAFETY: the caller guarantees the slot.
        unsafe { (at as *mut usize).write_unaligned(value) };
    }

    /// The screen context the stand-in `renderLevel` call passes. Resets the shader colour to `FRAME_COLOR`.
    pub fn screen() -> usize {
        let objects = Self::objects();
        // SAFETY: the shader colour block holds four floats and the dirty byte.
        unsafe {
            (objects.shader_color as *mut [f32; 4]).write(Self::FRAME_COLOR);
            ((objects.shader_color + 0x10) as *mut u8).write(0);
        }
        objects.screen
    }

    /// The wireframe `MaterialPtr` a player keeps at `LevelRendererCamera.mWireframeMaterial`: its first word.
    pub fn wireframe_info() -> usize {
        Self::objects().info
    }

    /// The `MaterialPtr` the stand-in item renderer holds, filled (`ready`) or empty (16 zero bytes).
    pub fn set_blend_ready(ready: bool) {
        let objects = Self::objects();
        let words = if ready { [objects.blend_info, 0xC0DE_0000] } else { [0, 0] };
        for (index, word) in words.into_iter().enumerate() {
            // SAFETY: the item renderer block is 200h bytes.
            unsafe { Self::put(objects.item_renderer + Self::BLEND_AT + index * 8, word) };
        }
    }

    /// `ClientInstance::getItemInHandRenderer`.
    pub extern "C" fn item_in_hand_renderer(_client: *mut std::ffi::c_void) -> usize {
        Self::record("item in hand renderer".to_string());
        Self::objects().item_renderer
    }

    /// Gives the stand-in camera one matrix on its view stack and one on its projection stack (`[view, projection]`), as
    /// two one-element deques; with `None` both stacks are empty.
    pub fn set_stacks(matrices: Option<[[[f32; 4]; 4]; 2]>) {
        let objects = Self::objects();
        let camera = objects.camera;
        for (slot, stack) in [0usize, 0x80].into_iter().enumerate() {
            let matrix = matrices.map(|[view, projection]| if slot == 0 { view } else { projection });
            let at = objects.camera_matrices + slot * 0x40;
            let size = usize::from(matrix.is_some());
            if let Some(matrix) = matrix {
                // SAFETY: the matrices block holds two 40h-byte slots.
                unsafe { (at as *mut [[f32; 4]; 4]).write(matrix) };
            }
            // The deque: proxy, map, map size, offset, size. The map is one pointer to the matrix.
            let map: &'static mut [usize; 1] = Box::leak(Box::new([at]));
            // SAFETY: the camera block is 200h bytes; each stack is five words.
            unsafe {
                Self::put(camera + stack + 8, std::ptr::from_mut(map) as usize);
                Self::put(camera + stack + 16, 1);
                Self::put(camera + stack + 24, 0);
                Self::put(camera + stack + 32, size);
            }
        }
    }

    /// Writes `bytes` into the stand-in `RenderMaterial`.
    pub fn set_material(bytes: [u8; 4]) {
        let material = Self::objects().material;
        for (at, value) in Self::MATERIAL_BYTES.into_iter().zip(bytes) {
            // SAFETY: the material block is 200h bytes.
            unsafe { ((material + at) as *mut u8).write(value) };
        }
    }

    /// The four patched bytes of the stand-in `RenderMaterial`.
    pub fn material() -> [u8; 4] {
        let material = Self::objects().material;
        // SAFETY: the material block is 200h bytes.
        Self::MATERIAL_BYTES.map(|at| unsafe { ((material + at) as *const u8).read() })
    }

    /// Whether a quad list is open in the stand-in tessellator.
    pub fn tessellating() -> bool {
        // SAFETY: the tessellator block is 300h bytes.
        unsafe { ((Self::objects().tessellator + Self::TESSELLATING) as *const u8).read() != 0 }
    }

    /// The shader colour and its dirty byte now.
    pub fn shader_color() -> ([f32; 4], u8) {
        let color = Self::objects().shader_color;
        // SAFETY: the shader colour block holds four floats and the dirty byte.
        unsafe { ((color as *const [f32; 4]).read(), ((color + 0x10) as *const u8).read()) }
    }

    /// The dirty byte of the stand-in camera world stack.
    pub fn camera_dirty() -> u8 {
        // SAFETY: the camera block is 200h bytes.
        unsafe { ((Self::objects().camera + Self::CAMERA_DIRTY) as *const u8).read() }
    }

    /// The translation of the last matrix popped.
    pub fn popped_translation() -> [f32; 3] {
        POPPED.lock().unwrap().map(f32::from_bits)
    }

    /// The positions, the colours and the mesh data box the tessellator held at the last `end`.
    pub fn last_end() -> Option<EndData> {
        LAST_END.lock().unwrap().clone()
    }

    /// The game calls so far.
    pub fn log() -> Vec<String> {
        LOG.lock().unwrap().clone()
    }

    fn record(line: String) {
        LOG.lock().unwrap().push(line);
    }

    /// The function entries of the mesh path, pointing to the stand-ins.
    pub fn entries() -> Vec<(&'static str, Resolved)> {
        let function = |address: usize| Resolved {
            kind: EntryKind::Function,
            value: address as u64 - Harness::BASE,
            recipes_agreeing: 2,
        };
        vec![
            ("Tessellator.begin", function(Self::begin as *const () as usize)),
            ("Tessellator.end", function(Self::end as *const () as usize)),
            ("Tessellator.clear", function(Self::clear as *const () as usize)),
            ("Mesh.isValid", function(Self::is_valid as *const () as usize)),
            ("Mesh.dtor", function(Self::dtor as *const () as usize)),
            ("Mesh.renderMeshTextureList", function(Self::render as *const () as usize)),
            ("MatrixStack.push", function(Self::push as *const () as usize)),
            ("MatrixStackRef.dtor", function(Self::pop as *const () as usize)),
            ("ActorShaderManager.setEntityConstants", function(Self::constants as *const () as usize)),
            ("BaseActorRenderer.ctor", function(Self::name_tag_renderer as *const () as usize)),
        ]
    }

    // The positions the tessellator holds: last - first, in vertices.
    //
    // Safety: `tessellator` is the stand-in tessellator.
    unsafe fn vertices(tessellator: usize) -> usize {
        // SAFETY: the vector words sit at `POSITIONS` in the block.
        let (first, last) = unsafe {
            (
                ((tessellator + Self::POSITIONS) as *const usize).read(),
                ((tessellator + Self::POSITIONS + 8) as *const usize).read(),
            )
        };
        (last - first) / 12
    }

    // The positions, the colours and the box of the mesh data.
    //
    // Safety: `tessellator` is the stand-in tessellator.
    unsafe fn snapshot(tessellator: usize) -> EndData {
        // SAFETY: the vector words sit inside the block; each vector holds `last - first` initialised elements.
        unsafe {
            let read = |at: usize| ((tessellator + at) as *const usize).read();
            let (first, last) = (read(Self::POSITIONS), read(Self::POSITIONS + 8));
            let positions = if first == 0 {
                Vec::new()
            } else {
                std::slice::from_raw_parts(first as *const [f32; 3], (last - first) / 12).to_vec()
            };
            let (first, last) = (read(Self::COLORS), read(Self::COLORS + 8));
            let colors = if first == 0 {
                Vec::new()
            } else {
                std::slice::from_raw_parts(first as *const u32, (last - first) / 4).to_vec()
            };
            let aabb = ((tessellator + Self::AABB) as *const [[f32; 3]; 2]).read_unaligned();
            (positions, colors, aabb)
        }
    }

    // Empties every vector of the mesh data, keeping their blocks (the stand-in heap leaks them).
    //
    // Safety: `tessellator` is the stand-in tessellator.
    unsafe fn empty(tessellator: usize) {
        for vector in 0..12 {
            let at = tessellator + 0x10 + vector * 0x18;
            // SAFETY: each vector is three words inside the block; last = first.
            unsafe { ((at + 8) as *mut usize).write((at as *const usize).read()) };
        }
    }

    // As on 70.26, the `DebugContextCallback` (an MSVC `std::function`, 40h bytes) comes by reference and its impl
    // pointer at 38h is read. Only an address in the caller's stack frame above this one is read.
    extern "C" fn begin(this: usize, callback: usize, mode: u8, reserve: i32, _flag: u8) {
        let here = std::ptr::from_ref(&callback) as usize;
        let in_caller_frame = callback > here && callback - here < 0x1_0000 && callback.is_multiple_of(8);
        let kind = if in_caller_frame {
            // SAFETY: the address is 8-aligned inside the live stack above this frame, which holds the 40h bytes.
            match unsafe { ((callback + 0x38) as *const usize).read() } {
                0 => "empty",
                _ => "set",
            }
        } else {
            "not_a_reference"
        };
        Self::record(format!("begin callback={kind} mode={mode} reserve={reserve}"));
        // SAFETY: `this` is the stand-in tessellator.
        unsafe { ((this + Self::TESSELLATING) as *mut u8).write(1) };
    }

    extern "C" fn end(this: usize, out: *mut u64, upload: i32, name: *const [usize; 2], mode: u16) -> *mut u64 {
        // SAFETY: `this` is the stand-in tessellator, `out` a mesh buffer, `name` a live view.
        unsafe {
            let vertices = Self::vertices(this);
            let count = ((this + Self::COUNT) as *const u32).read();
            let name = (*name)[1];
            *LAST_END.lock().unwrap() = Some(Self::snapshot(this));
            Self::record(format!("end upload={upload} vertices={vertices} count={count} name={name} mode={mode}"));
            out.write(vertices as u64);
            Self::empty(this);
            ((this + Self::TESSELLATING) as *mut u8).write(0);
        }
        out
    }

    extern "C" fn clear(this: usize) {
        Self::record("clear".to_string());
        // SAFETY: `this` is the stand-in tessellator; the list stays open, as the loader must close it.
        unsafe { Self::empty(this) };
    }

    extern "C" fn is_valid(mesh: *const u64) -> u8 {
        // SAFETY: `mesh` is a buffer `end` wrote.
        u8::from(unsafe { mesh.read() } > 0)
    }

    extern "C" fn dtor(mesh: *mut u64) {
        // SAFETY: `mesh` is a buffer `end` wrote.
        Self::record(format!("dtor vertices={}", unsafe { mesh.read() }));
    }

    #[allow(clippy::too_many_arguments)]
    extern "C" fn render(
        mesh: *const u64,
        context: usize,
        material: *const usize,
        list: *mut [u64; 41],
        unused_a: u32,
        unused_b: u32,
        capture: *const [u64; 8],
        unused_c: usize,
        optional: *const [u8; 0x40],
    ) {
        let objects = Self::objects();
        // SAFETY: the loader passes a mesh, a material, a list, a capture and an optional that live for the call.
        unsafe {
            let index = (*list)[4];
            let count = (*list)[40];
            let kind = if *material == objects.info {
                "wireframe"
            } else if *material == objects.blend_info {
                "blend"
            } else {
                "name_tag"
            };
            let atlas = (*list)[0] == Self::ATLAS_WORDS[0] as u64 && (*list)[2] == Self::ATLAS_WORDS[1] as u64;
            let held =
                if (*list)[1] == 0 { -1 } else { (*(((*list)[1] + 8) as *const AtomicI32)).load(Ordering::SeqCst) };
            let capture_bits = (*capture).iter().fold(0, |all, word| all | word);
            let optional_bits = (*optional)[0x30] | (*optional)[0x38];
            let color = (objects.shader_color as *const [f32; 4]).read();
            Self::record(format!(
                "render vertices={} context={:#x} material={kind} count={count} index={index} capture={capture_bits} optional={optional_bits} color={color:?} unused={unused_a},{unused_b},{unused_c} atlas={atlas} held={held}",
                mesh.read(),
                context - objects.screen,
            ));
            // The game destroys the list it was given: each `TexturePtr` drops the use of its two control blocks.
            if count >= 1 && index == 1 {
                for control in [(*list)[1], (*list)[3]] {
                    if control != 0 {
                        (*((control + 8) as *const AtomicI32)).fetch_sub(1, Ordering::SeqCst);
                    }
                }
            }
            (*list)[40] = 0;
        }
    }

    extern "C" fn push(this: usize, out: *mut [usize; 2], flag: u8) -> *mut [usize; 2] {
        Self::record(format!("push flag={flag}"));
        let matrix: &'static mut [[f32; 4]; 4] = Box::leak(Box::new([[0.0; 4]; 4]));
        for (index, column) in matrix.iter_mut().enumerate() {
            column[index] = 1.0;
        }
        // SAFETY: `out` is the 16-byte ref the loader passes.
        unsafe { out.write([this, std::ptr::from_mut(matrix) as usize]) };
        out
    }

    extern "C" fn pop(reference: *mut [usize; 2]) {
        // SAFETY: the ref `push` wrote; its matrix is leaked.
        let matrix = unsafe { &*((*reference)[1] as *const [[f32; 4]; 4]) };
        *POPPED.lock().unwrap() = [matrix[3][0], matrix[3][1], matrix[3][2]].map(f32::to_bits);
        Self::record("pop".to_string());
    }

    #[allow(clippy::too_many_arguments)]
    extern "C" fn constants(
        _entity: usize,
        _context: usize,
        _tile: *const [f32; 4],
        _tile_uv: *const [f32; 2],
        _block: *const [f32; 4],
        _overlay: *const [f32; 4],
        _change: *const [f32; 4],
        _change2: *const [f32; 4],
        _glint: *const [f32; 4],
        _glint_scale: *const [f32; 2],
        _uv_anim: *const [f32; 4],
        _a: f32,
        _b: f32,
        _c: f32,
        _d: f32,
    ) {
        Self::record("constants".to_string());
    }

    extern "C" fn name_tag_renderer(this: usize) -> usize {
        Self::record("name tag renderer".to_string());
        this
    }
}
