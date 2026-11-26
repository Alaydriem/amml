use crate::LoaderState;

/// The entries the item draw (`UiDraw::item`) calls the game with: the item renderer's draw function, the render
/// context built around it (constructor, destructor, size), the client's game and the item's animation frame.
///
/// `get_game_slot` and `animation_slot` are vtable slots (of the client and of the item); the rest are addresses and a
/// size.
#[derive(Clone, Copy, Debug)]
pub struct ItemDrawCalls {
    pub render_gui_item: usize,
    pub context_ctor: usize,
    pub context_dtor: usize,
    pub context_size: usize,
    pub get_game_slot: usize,
    pub animation_slot: usize,
    pub screen_context: usize,
}

impl ItemDrawCalls {
    /// The offset of the `ItemRenderer*` in a `BaseActorRenderContext`: the constructor stores it there on both gate
    /// builds.
    pub const ITEM_RENDERER_OFFSET: usize = 0x58;
    /// The smallest context that holds the item renderer.
    pub const MIN_CONTEXT_SIZE: usize = Self::ITEM_RENDERER_OFFSET + size_of::<usize>();
    /// The largest context accepted: a size above this is a wrong entry, not a game class.
    pub const MAX_CONTEXT_SIZE: usize = 0x1000;

    /// The calls from the resolved entries; `None` when one is missing or the context size is not a size.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let address = |name: &str| value(name).filter(|address| *address != 0);
        let calls = Self {
            render_gui_item: address("ItemRenderer.renderGuiItemNew")?,
            context_ctor: address("BaseActorRenderContext.ctor")?,
            context_dtor: address("BaseActorRenderContext.dtor")?,
            context_size: value("BaseActorRenderContext.size")?,
            get_game_slot: value("ClientInstance.getMinecraftGame")?,
            animation_slot: value("Item.getAnimationFrameFor")?,
            screen_context: value("MinecraftUIRenderContext.mScreenContext")?,
        };
        (Self::MIN_CONTEXT_SIZE..=Self::MAX_CONTEXT_SIZE).contains(&calls.context_size).then_some(calls)
    }
}
