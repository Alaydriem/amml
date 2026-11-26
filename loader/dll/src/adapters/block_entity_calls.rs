use crate::{
    ActorAdapter, BlockActorFactory, LoaderState, RenderLevelDetours, RenderMetadataLayout, StringByteInputLayout,
};

/// `NbtIo::read(IDataInput&)`: rcx the 8-byte result (the `unique_ptr<CompoundTag>`), rdx the input.
type NbtReadFn = unsafe extern "C" fn(*mut usize, *mut StringByteInputLayout) -> *mut usize;
/// `CompoundTag::~CompoundTag()`: rcx the tag.
type DtorFn = unsafe extern "C" fn(usize);
/// `static shared_ptr<BlockActor> BlockActor::loadStatic(BlockType const&, BlockPos const&, ILevel&, CompoundTag const&,
/// DataLoadHelper&)`: rcx the 16-byte result, rdx the type, r8 the position, r9 the level, then the tag and the helper on
/// the stack (survey section 7, row E2).
type LoadStaticFn =
    unsafe extern "C" fn(*mut [usize; 2], usize, *const [i32; 3], usize, usize, *mut usize) -> *mut [usize; 2];
/// `BlockActorRenderDispatcher::render`, the renderPos form, 12 arguments (survey section 7, row E4): rcx the
/// dispatcher, rdx the context, r8 the block source, r9 the component, then `Block const*`, `Vec3 const&` render
/// position, `BlockPos const&` world position, the alpha flag, `MaterialPtr const&` forced material, `ClientTexture
/// const*` forced texture, the breaking amount and the address of the `optional<RenderMetadata>`.
type DispatcherRenderFn = unsafe extern "C" fn(
    usize,
    usize,
    usize,
    usize,
    usize,
    *const [f32; 3],
    *const [i32; 3],
    u8,
    *const [u64; 2],
    usize,
    i32,
    *mut u64,
);

/// The game calls and offsets of `render.block_entities`, all resolved, for `BlockEntityAdapter`.
pub(crate) struct BlockEntityCalls {
    pub(crate) factory: BlockActorFactory,
    pub(crate) load_static: LoadStaticFn,
    pub(crate) nbt_read: NbtReadFn,
    pub(crate) tag_dtor: DtorFn,
    pub(crate) input_vtable: usize,
    pub(crate) helper_vtable: usize,
    pub(crate) render: DispatcherRenderFn,
    pub(crate) render_component_slot: usize,
    pub(crate) metadata: RenderMetadataLayout,
    pub(crate) dispatcher_at: usize,
    pub(crate) impl_at: usize,
    pub(crate) camera_target_at: usize,
    pub(crate) level_at: usize,
    pub(crate) block_type_at: usize,
    pub(crate) position_at: usize,
}

impl BlockEntityCalls {
    /// The calls of the running build, or the first entry (or adapter) that is missing. Installs the shared
    /// `renderLevel` detour, which releases the actors.
    pub(crate) fn new(state: &'static LoaderState) -> Result<Self, &'static str> {
        let value = |name: &'static str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let address = |name: &'static str| value(name).filter(|value| *value != 0).ok_or(name);
        let metadata = state
            .resolve("RenderMetadata.engaged")
            .ok()
            .and_then(RenderMetadataLayout::new)
            .ok_or("RenderMetadata.engaged")?;
        let factory = BlockActorFactory::new(state).ok_or("VanillaBlockActorFactory.createBlockActor")?;
        if ActorAdapter::get().is_none() {
            return Err("actor adapter");
        }
        if !RenderLevelDetours::install(state) {
            return Err(RenderLevelDetours::RENDER_LEVEL);
        }
        let load_static = address("BlockActor.loadStatic")?;
        let nbt_read = address("NbtIo.read")?;
        let tag_dtor = address("CompoundTag.dtor")?;
        let render = address("BlockActorRenderDispatcher.render")?;
        // SAFETY: each address is the derived function of its entry, whose signature is the type it becomes.
        let (load_static, nbt_read, tag_dtor, render) = unsafe {
            (
                std::mem::transmute::<usize, LoadStaticFn>(load_static),
                std::mem::transmute::<usize, NbtReadFn>(nbt_read),
                std::mem::transmute::<usize, DtorFn>(tag_dtor),
                std::mem::transmute::<usize, DispatcherRenderFn>(render),
            )
        };
        Ok(Self {
            factory,
            load_static,
            nbt_read,
            tag_dtor,
            input_vtable: address("StringByteInput.vtable")?,
            helper_vtable: address("DefaultDataLoadHelper.vtable")?,
            render,
            render_component_slot: value("BlockActor._getRenderComponent").ok_or("BlockActor._getRenderComponent")?,
            metadata,
            dispatcher_at: value("BaseActorRenderContext.mBlockEntityRenderDispatcher")
                .ok_or("BaseActorRenderContext.mBlockEntityRenderDispatcher")?,
            impl_at: value("BaseActorRenderContext.mImpl").ok_or("BaseActorRenderContext.mImpl")?,
            camera_target_at: value("BaseActorRenderContext.Impl.mCameraTargetPosition")
                .ok_or("BaseActorRenderContext.Impl.mCameraTargetPosition")?,
            level_at: address("Actor.mLevel")?,
            block_type_at: value("Block.mBlockType").ok_or("Block.mBlockType")?,
            position_at: value("BlockActor.mPosition").ok_or("BlockActor.mPosition")?,
        })
    }
}
