use amml_game::{
    CompoundTagRef, ContainerScreenController, EntryId, EntryNeed, ItemRenderer, UiRenderContext, UiScene,
};

/// The entries of the shulker box preview probe: the five game features of `amml-game` and the font handle.
pub struct PeekEntries;

impl PeekEntries {
    /// The slot of `ClientInstance::getFontHandle` in the client vtable.
    pub const GET_FONT_HANDLE: EntryId = EntryId::new("ClientInstance.getFontHandle");
    /// The non-deleting destructor of `FontHandle`.
    pub const FONT_HANDLE_DTOR: EntryId = EntryId::new("FontHandle.dtor");
    /// The size of a `FontHandle`.
    pub const FONT_HANDLE_SIZE: EntryId = EntryId::new("FontHandle.size");

    /// The feature of the font entries.
    pub const FONT_FEATURE: &'static str = "peek.font";

    /// Every entry, each under the feature of its group.
    pub fn wanted() -> Vec<(EntryId, EntryNeed)> {
        let groups: [(&'static str, &[EntryId]); 6] = [
            (UiRenderContext::FEATURE, &UiRenderContext::ENTRIES),
            (ContainerScreenController::FEATURE, &ContainerScreenController::ENTRIES),
            (ItemRenderer::FEATURE, &ItemRenderer::ENTRIES),
            (CompoundTagRef::FEATURE, &CompoundTagRef::ENTRIES),
            (
                UiScene::FEATURE,
                &[EntryId::UI_SCENE_VTABLE, EntryId::UI_SCENE_HANDLE_DIRECTION, EntryId::UI_SCENE_SCREEN_VIEW],
            ),
            (Self::FONT_FEATURE, &[Self::GET_FONT_HANDLE, Self::FONT_HANDLE_DTOR, Self::FONT_HANDLE_SIZE]),
        ];
        groups
            .into_iter()
            .flat_map(|(feature, ids)| ids.iter().map(move |id| (*id, EntryNeed::Optional { feature })))
            .collect()
    }
}
