use std::hash::{Hash, Hasher};

use amml_ui::id::IdScope;

/// The egui ids of the amml-studio-ui components, used with `Id::unique`. Each component task adds
/// the ids its component needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StudioUiId {
    /// The inspector tab strip (its underline slides).
    InspectorTabs,
    /// The inspector drawer's area at the small breakpoint.
    InspectorDrawer,
    /// The easing popover's family filter (its selection slides).
    EasingFamily,
    /// The dialogs' modals (egui keeps which modal is on top).
    ExportDialog,
    RenderQueue,
    RenderProgress,
    CoveragePanel,
    ConfirmDialog,
    RenameDialog,
    /// The export dialog's format, resolution and frame rate controls (their selections slide).
    ExportFormat,
    ExportResolution,
    ExportFrameRate,
    /// The text fields (egui keeps their cursor and focus).
    CoverageSearch,
    RenameField,
    /// What a drag on the lanes started as (a key move or a box), kept until the drag ends.
    TimelineDrag,
    /// The scroll areas that keep their offset: the easing presets, the saved easings and the undo history.
    EasingGrid,
    EasingSaved,
    UndoHistory,
    /// The capture rail's Discard question.
    DiscardPopover,
    /// The Library app bar's tabs and buttons, and the hero card's button.
    LibraryAppBar,
    HeroCard,
    /// The scene switcher list's rows.
    SceneSwitcherList,
    /// The source list's remove buttons and its add button.
    SourceList,
    /// The placeholder picture's texture, kept in the context's temporary data.
    ReplayPlaceholder,
}

impl Hash for StudioUiId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        IdScope::hash_type::<Self, H>(state);
        std::mem::discriminant(self).hash(state);
    }
}
