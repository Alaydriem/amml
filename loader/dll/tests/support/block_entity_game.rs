use std::collections::HashSet;
use std::hint::black_box;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, AtomicU8, AtomicUsize, Ordering};

use amml::{LoaderState, StringByteInputLayout};
use amml_data::{EntryKind, Resolved};

use super::{ActorGame, FakeVtable, Harness, RenderGame};

// The game calls of the block entity path, one line each, in order.
static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
// The `RenderMetadata.engaged` value the stand-in dispatcher reads the optional with.
static ENGAGED: AtomicUsize = AtomicUsize::new(0);
// The byte `Level.isClientSide` gives.
static CLIENT_SIDE: AtomicU8 = AtomicU8::new(1);
// The vtables of the two level renderer cameras: the local player's and another one. No slot runs.
static PLAYER_RENDERER_VTABLE: FakeVtable<1> = FakeVtable([BlockEntityGame::unused as *const ()]);
static OTHER_RENDERER_VTABLE: FakeVtable<1> = FakeVtable([BlockEntityGame::unused as *const ()]);
// The vtable of every stand-in block actor: slot 10h `isType`, slot 15h `_getRenderComponent`.
static ACTOR_VTABLE: FakeVtable<0x16> = {
    let mut slots = [BlockEntityGame::unused as *const (); 0x16];
    slots[0x10] = BlockEntityGame::is_type as *const ();
    slots[0x15] = BlockEntityGame::render_component as *const ();
    FakeVtable(slots)
};
// The vtable of every stand-in control block: `_Destroy`, `_Delete_this`.
static CONTROL_VTABLE: FakeVtable<2> =
    FakeVtable([BlockEntityGame::control_destroy as *const (), BlockEntityGame::control_delete as *const ()]);
// The Level vtable: slot 3 is the flag slot (`Level.isClientSide`).
static LEVEL_VTABLE: FakeVtable<4> = FakeVtable([
    BlockEntityGame::unused as *const (),
    BlockEntityGame::unused as *const (),
    BlockEntityGame::unused as *const (),
    BlockEntityGame::is_client_side as *const (),
]);
static LEVEL: VtableObject<4> = VtableObject { vtable: &LEVEL_VTABLE };
static PLAYER_RENDERER: VtableObject<1> = VtableObject { vtable: &PLAYER_RENDERER_VTABLE };
static OTHER_RENDERER: VtableObject<1> = VtableObject { vtable: &OTHER_RENDERER_VTABLE };
static INPUT_VTABLE: [usize; 11] = [0; 11];
static HELPER_VTABLE: [usize; 18] = [0; 18];
// The dispatcher: the skull renderer at 20h (`BlockActorRenderDispatcher.mSkullRenderer`).
static SKULL_RENDERER: [u64; 4] = [0; 4];
static DISPATCHER: DispatcherObject = DispatcherObject { _head: [0; 4], skull: &SKULL_RENDERER, _tail: [0; 3] };
// The stand-in block types: a chest, a sign and a skull; their blocks hold the type at `BLOCK_TYPE_AT`, and the full name
// of a type is at `FULL_NAME_AT` in it.
static CHEST_TYPE: [u64; 4] = [0; 4];
static SIGN_TYPE: [u64; 4] = [0; 4];
static SKULL_TYPE: [u64; 4] = [0; 4];
static CHEST_BLOCK: BlockObject = BlockObject { _head: [0; 13], block_type: &CHEST_TYPE };
static SIGN_BLOCK: BlockObject = BlockObject { _head: [0; 13], block_type: &SIGN_TYPE };
static SKULL_BLOCK: BlockObject = BlockObject { _head: [0; 13], block_type: &SKULL_TYPE };
// The camera of the last `context()`, whose world matrix stack is at 40h; the matrix the last push gave; the stack top
// each `renderSkullUpright` saw.
static CAMERA_OBJECT: AtomicUsize = AtomicUsize::new(0);
static TOP: AtomicUsize = AtomicUsize::new(0);
static UPRIGHT_TOPS: Mutex<Vec<Matrix>> = Mutex::new(Vec::new());

type Matrix = [[f32; 4]; 4];
// The actors made so far, in order.
static ACTORS: Mutex<Vec<usize>> = Mutex::new(Vec::new());
static CAMERA: [f32; 3] = [1.0, 64.0, 2.0];

// A stand-in game object that is only its vtable pointer.
#[repr(C)]
struct VtableObject<const N: usize> {
    vtable: &'static FakeVtable<N>,
}

// A stand-in `BlockActorRenderDispatcher`: the skull renderer pointer at 20h.
#[repr(C)]
struct DispatcherObject {
    _head: [u64; 4],
    skull: &'static [u64; 4],
    _tail: [u64; 3],
}

// A stand-in `Block`: the block type pointer at 68h.
#[repr(C)]
struct BlockObject {
    _head: [u64; 13],
    block_type: &'static [u64; 4],
}

/// A stand-in block actor: its vtable, `mPosition` at 8h, the actor kind (2 for a chest) at 14h, the chest pair flags
/// at 40h, the pair pointer at 48h, the pair position at 50h, and the render component at 60h.
#[repr(C)]
pub struct ActorObject {
    vtable: &'static FakeVtable<0x16>,
    pub position: [i32; 3],
    pub kind: u8,
    _gap: [u8; 0x2B],
    pub flags: u8,
    _gap2: [u8; 7],
    pub paired: usize,
    pub paired_position: [i32; 3],
    _gap3: [u8; 4],
    component: [u64; 4],
}

// A stand-in `_Ref_count_base`: vtable, uses, weaks, then the actor it owns.
#[repr(C)]
struct ControlObject {
    vtable: &'static FakeVtable<2>,
    uses: AtomicI32,
    weaks: AtomicI32,
    actor: usize,
}

/// Stand-ins with the game signatures for the block entity ghosts: the shared block entity body, the two level renderer
/// camera vtables, the factory in both forms, `loadStatic`, `NbtIo::read` and the tag destructor, the block actor
/// virtuals (`isType`, `_getRenderComponent`), the control block release, the level flag slot and
/// `BlockActorRenderDispatcher::render`, over the local player of `ActorGame` and the `renderLevel` of `RenderGame`.
pub struct BlockEntityGame;

impl BlockEntityGame {
    pub const CHEST_PAIR_FLAGS_AT: u64 = 0x40;
    pub const CHEST_PAIRED_AT: u64 = 0x48;
    pub const CHEST_PAIRED_POSITION_AT: u64 = 0x50;
    const BLOCK_TYPE_AT: u64 = 0x68;
    const COMPONENT_AT: usize = 0x60;
    const DISPATCHER_AT: usize = 0x30;
    const IMPL_AT: usize = 0xA8;
    const CAMERA_AT: usize = 0x20;
    const TAG_SIZE: usize = 0x18;
    const SKULL_RENDERER_AT: u64 = 0x20;
    const FULL_NAME_AT: usize = 0x10;
    const SCREEN_AT: usize = 0x28;
    const SCREEN_CAMERA_AT: usize = 0x18;
    const WORLD_STACK_AT: usize = 0x40;
    const STACK_DIRTY_AT: usize = 0x38;
    /// `BlockActorType::Skull`.
    pub const SKULL_KIND: u8 = 6;

    /// The actor entries, the `renderLevel` entries of `RenderGame` that are not among them and the block entity entries,
    /// with the 26.63 factory (or the 70.26 factory and the level flag slot with `flag_form`) and `RenderMetadata.engaged`
    /// = `engaged`; `without` names entries to leave out. Clears the recorded calls; the local player's level is the
    /// stand-in level.
    pub fn entries(without: &[&str], flag_form: bool, engaged: u64) -> Vec<(&'static str, Resolved)> {
        let mut entries = ActorGame::entries(without);
        let present: HashSet<&str> = entries.iter().map(|(name, _)| *name).collect();
        let render: Vec<(&'static str, Resolved)> =
            RenderGame::entries(without).into_iter().filter(|(name, _)| !present.contains(name)).collect();
        entries.extend(render);
        ENGAGED.store(engaged as usize, Ordering::SeqCst);
        ActorGame::set_level(std::ptr::from_ref(&LEVEL).addr());
        let function = |address: usize| (EntryKind::Function, address as u64 - Harness::BASE);
        let global = |address: usize| (EntryKind::Global, address as u64 - Harness::BASE);
        let factory = if flag_form {
            (
                "VanillaBlockActorFactory.createBlockActorWithFlag",
                function(Self::create_with_flag as *const () as usize),
            )
        } else {
            ("VanillaBlockActorFactory.createBlockActor", function(Self::create as *const () as usize))
        };
        let mut more = vec![
            ("LevelRendererCamera.renderBlockEntitiesBody", function(Self::body as *const () as usize)),
            ("LevelRendererPlayer.vtable", global(std::ptr::from_ref(&PLAYER_RENDERER_VTABLE).addr())),
            ("BlockActor.loadStatic", function(Self::load_static as *const () as usize)),
            ("BlockActor._getRenderComponent", (EntryKind::VSlot, 0x15)),
            ("BlockActor.isType", (EntryKind::VSlot, 0x10)),
            ("BlockActor.mPosition", (EntryKind::Field, 8)),
            ("DefaultDataLoadHelper.vtable", global(HELPER_VTABLE.as_ptr().addr())),
            ("BlockActorRenderDispatcher.render", function(Self::render as *const () as usize)),
            ("RenderMetadata.engaged", (EntryKind::Field, engaged)),
            ("BaseActorRenderContext.mBlockEntityRenderDispatcher", (EntryKind::Field, Self::DISPATCHER_AT as u64)),
            ("BaseActorRenderContext.mImpl", (EntryKind::Field, Self::IMPL_AT as u64)),
            ("BaseActorRenderContext.Impl.mCameraTargetPosition", (EntryKind::Field, Self::CAMERA_AT as u64)),
            ("NbtIo.read", function(Self::nbt_read as *const () as usize)),
            ("StringByteInput.vtable", global(INPUT_VTABLE.as_ptr().addr())),
            ("CompoundTag.dtor", function(Self::tag_dtor as *const () as usize)),
            ("Actor.mLevel", (EntryKind::Field, ActorGame::LEVEL_AT)),
            ("Block.mBlockType", (EntryKind::Field, Self::BLOCK_TYPE_AT)),
            ("ChestBlockActor.mPairFlags", (EntryKind::Field, Self::CHEST_PAIR_FLAGS_AT)),
            ("ChestBlockActor.mLargeChestPaired", (EntryKind::Field, Self::CHEST_PAIRED_AT)),
            ("ChestBlockActor.mLargeChestPairedPosition", (EntryKind::Field, Self::CHEST_PAIRED_POSITION_AT)),
            ("BlockActorRenderDispatcher.mSkullRenderer", (EntryKind::Field, Self::SKULL_RENDERER_AT)),
            ("BlockType.mNameInfo.mFullName", (EntryKind::Field, Self::FULL_NAME_AT as u64)),
            ("HashedString.layout", (EntryKind::Field, 0)),
            factory,
        ];
        if flag_form {
            more.extend([
                ("Level.isClientSide", (EntryKind::VSlot, 3)),
                ("SkullBlockRenderer.renderSkullUpright", function(Self::render_skull_upright as *const () as usize)),
                ("MatrixStack.push", function(Self::push_matrix as *const () as usize)),
                ("MatrixStackRef.dtor", function(Self::pop_matrix as *const () as usize)),
                ("ScreenContext.camera", (EntryKind::Field, Self::SCREEN_CAMERA_AT as u64)),
                ("Camera.mForward", (EntryKind::Field, 0x118)),
            ]);
        } else {
            more.push(("SkullBlockRenderer.renderSkull", function(Self::render_skull as *const () as usize)));
        }
        for (name, (kind, value)) in more {
            if !without.contains(&name) {
                entries.retain(|(known, _)| *known != name);
                entries.push((name, Resolved { kind, value, recipes_agreeing: 2 }));
            }
        }
        LOG.lock().unwrap().clear();
        ACTORS.lock().unwrap().clear();
        UPRIGHT_TOPS.lock().unwrap().clear();
        entries
    }

    /// The game calls so far, one line each.
    pub fn log() -> Vec<String> {
        LOG.lock().unwrap().clone()
    }

    /// The address of the stand-in chest block and of the stand-in sign block.
    pub fn chest_block() -> usize {
        std::ptr::from_ref(&CHEST_BLOCK).addr()
    }

    pub fn sign_block() -> usize {
        std::ptr::from_ref(&SIGN_BLOCK).addr()
    }

    pub fn skull_block() -> usize {
        std::ptr::from_ref(&SKULL_BLOCK).addr()
    }

    /// The stack top each `renderSkullUpright` call saw, in order.
    pub fn upright_tops() -> Vec<Matrix> {
        UPRIGHT_TOPS.lock().unwrap().clone()
    }

    /// The actors the factory and `loadStatic` made so far, in order.
    pub fn actors() -> Vec<&'static ActorObject> {
        // SAFETY: each address is a leaked `ActorObject`.
        ACTORS.lock().unwrap().iter().map(|address| unsafe { &*std::ptr::with_exposed_provenance(*address) }).collect()
    }

    /// The byte the level flag slot gives.
    pub fn set_client_side(value: u8) {
        CLIENT_SIDE.store(value, Ordering::SeqCst);
    }

    /// A stand-in `BaseActorRenderContext`: the `ScreenContext` at 28h (its camera at 18h), the dispatcher at 30h and
    /// the `Impl` at A8h, whose camera target is (1, 64, 2).
    pub fn context() -> usize {
        let implementation: &'static mut [f32; 16] = Box::leak(Box::new([0.0; 16]));
        implementation[Self::CAMERA_AT / 4..Self::CAMERA_AT / 4 + 3].copy_from_slice(&CAMERA);
        let camera: &'static mut [u64; 0x24] = Box::leak(Box::new([0; 0x24]));
        CAMERA_OBJECT.store(std::ptr::from_mut(camera).addr(), Ordering::SeqCst);
        let screen: &'static mut [usize; 4] = Box::leak(Box::new([0; 4]));
        screen[Self::SCREEN_CAMERA_AT / 8] = std::ptr::from_mut(camera).addr();
        let context: &'static mut [usize; 0x18] = Box::leak(Box::new([0; 0x18]));
        context[Self::SCREEN_AT / 8] = std::ptr::from_mut(screen).addr();
        context[Self::DISPATCHER_AT / 8] = std::ptr::from_ref(&DISPATCHER).addr();
        context[Self::IMPL_AT / 8] = std::ptr::from_mut(implementation).addr();
        std::ptr::from_mut(context).addr()
    }

    /// The local player's level renderer camera and another camera.
    pub fn player_renderer() -> usize {
        std::ptr::from_ref(&PLAYER_RENDERER).addr()
    }

    pub fn other_renderer() -> usize {
        std::ptr::from_ref(&OTHER_RENDERER).addr()
    }

    /// Calls (through its address) the shared block entity body with `this`, `context` and the alpha flag.
    pub fn pass(this: usize, context: usize, alpha: bool) {
        let function = black_box(Self::body as extern "C" fn(usize, usize, u8));
        function(black_box(this), black_box(context), black_box(u8::from(alpha)));
    }

    fn push(line: String) {
        LOG.lock().unwrap().push(line);
    }

    fn type_name(block_type: usize) -> &'static str {
        if block_type == CHEST_TYPE.as_ptr().addr() {
            "chest"
        } else if block_type == SIGN_TYPE.as_ptr().addr() {
            "sign"
        } else if block_type == SKULL_TYPE.as_ptr().addr() {
            "skull"
        } else {
            "other"
        }
    }

    // A new actor of the block type at `block_type` at `pos`, written to `out` with a fresh control block.
    fn make(out: *mut [usize; 2], block_type: usize, pos: [i32; 3]) {
        let kind = match Self::type_name(block_type) {
            "chest" => 2,
            "skull" => Self::SKULL_KIND,
            _ => 7,
        };
        let actor = Box::leak(Box::new(ActorObject {
            vtable: &ACTOR_VTABLE,
            position: pos,
            kind,
            _gap: [0; 0x2B],
            flags: 0x5,
            _gap2: [0; 7],
            paired: 0,
            paired_position: [0; 3],
            _gap3: [0; 4],
            component: [0; 4],
        }));
        let actor = std::ptr::from_mut(actor).addr();
        let control = Box::leak(Box::new(ControlObject {
            vtable: &CONTROL_VTABLE,
            uses: AtomicI32::new(1),
            weaks: AtomicI32::new(1),
            actor,
        }));
        ACTORS.lock().unwrap().push(actor);
        // SAFETY: `out` is the loader's 16-byte result.
        unsafe { out.write([actor, std::ptr::from_mut(control).addr()]) };
    }

    fn actor_index(actor: usize) -> usize {
        ACTORS.lock().unwrap().iter().position(|known| *known == actor).unwrap_or(usize::MAX)
    }

    #[inline(never)]
    extern "C" fn body(this: usize, context: usize, alpha: u8) {
        black_box((this, context, alpha));
        Self::push(format!("body alpha={alpha}"));
    }

    // `createBlockActor(out, pos, type)`, the 26.63 form.
    extern "C" fn create(out: *mut [usize; 2], pos: *const [i32; 3], block_type: usize) -> *mut [usize; 2] {
        // SAFETY: the loader passes a live position.
        let pos = unsafe { pos.read() };
        Self::push(format!("create type={} pos={},{},{}", Self::type_name(block_type), pos[0], pos[1], pos[2]));
        Self::make(out, block_type, pos);
        out
    }

    // `createBlockActor(out, pos, flag, type)`, the 70.26 form.
    extern "C" fn create_with_flag(
        out: *mut [usize; 2],
        pos: *const [i32; 3],
        flag: u8,
        block_type: usize,
    ) -> *mut [usize; 2] {
        // SAFETY: the loader passes a live position.
        let pos = unsafe { pos.read() };
        Self::push(format!(
            "create type={} pos={},{},{} flag={flag}",
            Self::type_name(block_type),
            pos[0],
            pos[1],
            pos[2]
        ));
        Self::make(out, block_type, pos);
        out
    }

    // `loadStatic(out, type, pos, level, tag, helper)`.
    extern "C" fn load_static(
        out: *mut [usize; 2],
        block_type: usize,
        pos: *const [i32; 3],
        level: usize,
        tag: usize,
        helper: *const usize,
    ) -> *mut [usize; 2] {
        // SAFETY: the loader passes a live position and a live helper object.
        let (pos, helper) = unsafe { (pos.read(), helper.read()) };
        // SAFETY: the tag is the stand-in tag of `nbt_read`, whose first word is the length of the bytes it read.
        let length = unsafe { std::ptr::with_exposed_provenance::<usize>(tag).read() };
        Self::push(format!(
            "load type={} pos={},{},{} level={} tag={length} helper={}",
            Self::type_name(block_type),
            pos[0],
            pos[1],
            pos[2],
            level == std::ptr::from_ref(&LEVEL).addr(),
            helper == HELPER_VTABLE.as_ptr().addr()
        ));
        Self::make(out, block_type, pos);
        out
    }

    // `NbtIo::read(IDataInput&)`: a tag on the game heap whose first word is the length of the input.
    extern "C" fn nbt_read(out: *mut usize, input: *mut StringByteInputLayout) -> *mut usize {
        // SAFETY: the loader passes a live input over its own bytes.
        let input = unsafe { &*input };
        assert_eq!(input.vtable.addr(), INPUT_VTABLE.as_ptr().addr());
        let tag = LoaderState::get().unwrap().heap().alloc(Self::TAG_SIZE).addr();
        // SAFETY: the heap gave `TAG_SIZE` writable bytes.
        unsafe { std::ptr::with_exposed_provenance_mut::<usize>(tag).write_unaligned(input.len) };
        Self::push(format!("nbt read len={}", input.len));
        // SAFETY: `out` is the loader's 8-byte result.
        unsafe { out.write(tag) };
        out
    }

    extern "C" fn tag_dtor(_tag: usize) {
        Self::push("tag dtor".to_string());
    }

    // `bool BlockActor::isType(BlockActorType) const`: true when the type is the actor's kind.
    extern "C" fn is_type(this: *const ActorObject, kind: u8) -> u8 {
        // SAFETY: the loader calls the slot on a live stand-in actor.
        let actor = unsafe { &*this };
        Self::push(format!("is_type actor={} type={kind}", Self::actor_index(this.addr())));
        u8::from(actor.kind == kind)
    }

    extern "C" fn render_component(this: usize) -> usize {
        this + Self::COMPONENT_AT
    }

    extern "C" fn control_destroy(this: *mut ControlObject) {
        // SAFETY: the loader released the last use of a live stand-in control block.
        let actor = unsafe { (*this).actor };
        Self::push(format!("release actor={}", Self::actor_index(actor)));
        // SAFETY: the control block's actor is a live stand-in actor until this call returns.
        let paired = unsafe { (*std::ptr::with_exposed_provenance::<ActorObject>(actor)).paired };
        Self::push(format!("paired at release actor={} paired={}", Self::actor_index(actor), paired != 0));
    }

    extern "C" fn control_delete(_this: *mut ControlObject) {}

    extern "C" fn is_client_side(_this: usize) -> u8 {
        CLIENT_SIDE.load(Ordering::SeqCst)
    }

    // `BlockActorRenderDispatcher::render`, the 12 arguments of the renderPos form.
    #[allow(clippy::too_many_arguments)]
    extern "C" fn render(
        dispatcher: usize,
        context: usize,
        source: usize,
        component: usize,
        block: usize,
        render_pos: *const [f32; 3],
        world_pos: *const [i32; 3],
        alpha: u8,
        material: *const [u64; 2],
        texture: usize,
        breaking: i32,
        metadata: *const u8,
    ) {
        let engaged = ENGAGED.load(Ordering::SeqCst);
        // SAFETY: the loader passes live buffers: two positions, a 16-byte material and an optional that covers the
        // engaged byte.
        let (render_pos, world_pos, material, flag) =
            unsafe { (render_pos.read(), world_pos.read(), material.read(), metadata.add(engaged).read()) };
        // SAFETY: the context is the stand-in of `context()`.
        let expected_dispatcher = unsafe { std::ptr::with_exposed_provenance::<usize>(context + 0x30).read() };
        let actor = component - Self::COMPONENT_AT;
        Self::push(format!(
            "render dispatcher={} source={} actor={} block={} pos={},{},{} world={},{},{} alpha={alpha} material={},{} \
             texture={texture} breaking={breaking} engaged@{engaged:#x}={flag}",
            dispatcher == expected_dispatcher,
            source == ActorGame::block_source(),
            Self::actor_index(actor),
            if block == Self::chest_block() { "chest" } else { "sign" },
            render_pos[0],
            render_pos[1],
            render_pos[2],
            world_pos[0],
            world_pos[1],
            world_pos[2],
            material[0],
            material[1],
        ));
    }

    // The type whose full name is at `name`.
    fn name_of(name: usize) -> &'static str {
        name.checked_sub(Self::FULL_NAME_AT).map_or("other", Self::type_name)
    }

    // Whether the first `RenderMetadata.engaged` bytes at `metadata` (the size of a `RenderMetadata`) are all zero.
    fn metadata_is_empty(metadata: *const u8) -> bool {
        let size = ENGAGED.load(Ordering::SeqCst);
        // SAFETY: the loader passes a buffer that covers a `RenderMetadata`.
        unsafe { std::slice::from_raw_parts(metadata, size) }.iter().all(|byte| *byte == 0)
    }

    fn expected_context(context: usize) -> bool {
        // SAFETY: the context is the stand-in of `context()`.
        let dispatcher = unsafe { std::ptr::with_exposed_provenance::<usize>(context + Self::DISPATCHER_AT).read() };
        dispatcher == std::ptr::from_ref(&DISPATCHER).addr()
    }

    // `SkullBlockRenderer::renderSkull`, 26.63: this, context, metadata, position, face, rotation in degrees, name,
    // material, texture, breaking, animation, glint.
    #[allow(clippy::too_many_arguments)]
    extern "C" fn render_skull(
        this: usize,
        context: usize,
        metadata: *const u8,
        pos: *const [f32; 3],
        face: i32,
        rotation: f32,
        name: usize,
        material: usize,
        texture: usize,
        breaking: i32,
        animation: f32,
        glint: u8,
    ) {
        // SAFETY: the loader passes a live position.
        let pos = unsafe { pos.read() };
        Self::push(format!(
            "skull renderer={} context={} metadata_empty={} pos={},{},{} face={face} rotation={rotation} name={} \
             material={material} texture={texture} breaking={breaking} animation={animation} glint={glint}",
            this == SKULL_RENDERER.as_ptr().addr(),
            Self::expected_context(context),
            Self::metadata_is_empty(metadata),
            pos[0],
            pos[1],
            pos[2],
            Self::name_of(name),
        ));
    }

    // `SkullBlockRenderer::renderSkullUpright`, 70.26: this, context, metadata, position, name, animation, glint. Keeps
    // the stack top it sees.
    extern "C" fn render_skull_upright(
        this: usize,
        context: usize,
        metadata: *const u8,
        pos: *const [f32; 3],
        name: usize,
        animation: f32,
        glint: u8,
    ) {
        // SAFETY: the loader passes a live position.
        let pos = unsafe { pos.read() };
        let stack = CAMERA_OBJECT.load(Ordering::SeqCst) + Self::WORLD_STACK_AT;
        // SAFETY: the stack of the stand-in camera; the dirty flag is a byte in it.
        let dirty = unsafe { std::ptr::with_exposed_provenance::<u8>(stack + Self::STACK_DIRTY_AT).read() };
        // SAFETY: the matrix the last push gave, leaked.
        let top = unsafe { std::ptr::with_exposed_provenance::<Matrix>(TOP.load(Ordering::SeqCst)).read() };
        UPRIGHT_TOPS.lock().unwrap().push(top);
        Self::push(format!(
            "upright renderer={} context={} metadata_empty={} pos={},{},{} name={} animation={animation} glint={glint} \
             dirty={dirty}",
            this == SKULL_RENDERER.as_ptr().addr(),
            Self::expected_context(context),
            Self::metadata_is_empty(metadata),
            pos[0],
            pos[1],
            pos[2],
            Self::name_of(name),
        ));
    }

    // `MatrixStack::push(bool)`: an identity matrix as the new top.
    extern "C" fn push_matrix(this: usize, out: *mut [usize; 2], flag: u8) -> *mut [usize; 2] {
        let camera_stack = this == CAMERA_OBJECT.load(Ordering::SeqCst) + Self::WORLD_STACK_AT;
        Self::push(format!("push camera_stack={camera_stack} flag={flag}"));
        let mut matrix: Matrix = [[0.0; 4]; 4];
        for (index, column) in matrix.iter_mut().enumerate() {
            column[index] = 1.0;
        }
        let matrix = Box::leak(Box::new(matrix));
        TOP.store(std::ptr::from_mut(matrix).addr(), Ordering::SeqCst);
        // SAFETY: `out` is the 16-byte ref the loader passes.
        unsafe { out.write([this, std::ptr::from_mut(matrix).addr()]) };
        out
    }

    extern "C" fn pop_matrix(_reference: *mut [usize; 2]) {
        Self::push("pop".to_string());
    }

    extern "C" fn unused() {
        panic!("a stand-in slot that no test calls");
    }
}
