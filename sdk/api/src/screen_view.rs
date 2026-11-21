use std::marker::PhantomData;

use amml_abi::{
    AmmlInputMode, AmmlInventoryTable, AmmlPoint, AmmlRect, AmmlScreenKind, AmmlSize, AmmlUiProfile, AmmlUiTable,
};

use crate::{ContainerScreen, InputMode, Point, Rect, ScreenId, ScreenKind, Size, UiProfile, ViewId};

/// One screen view of the game, as `ScreenRenderExt::view` gives it, for the call of the listener that got the
/// event. A view cannot outlive that call by its lifetime; its id kept longer is refused by the loader.
///
/// Every read goes through the loader, which checks that the call of the event still runs on this thread. A read
/// it cannot make gives `None`. Sizes and positions are in GUI units, the units of the UI layout of the game.
pub struct ScreenView<'f> {
    table: AmmlUiTable,
    inventory: Option<AmmlInventoryTable>,
    module: &'static str,
    view: ViewId,
    _event: PhantomData<&'f ()>,
}

impl<'f> ScreenView<'f> {
    pub(crate) fn new(
        table: AmmlUiTable,
        inventory: Option<AmmlInventoryTable>,
        module: &'static str,
        view: ViewId,
    ) -> Self {
        Self { table, inventory, module, view, _event: PhantomData }
    }

    /// The kind of the screen; `Other` for a screen this crate does not tell apart and when it cannot be read.
    pub fn kind(&self) -> ScreenKind {
        let mut out = AmmlScreenKind::OTHER;
        // SAFETY: the loader checks the view id before it reads game memory, and `out` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { (self.table.view_kind)(self.view.0, &mut out) };
        if status.is_ok() { ScreenKind::from(out) } else { ScreenKind::Other }
    }

    /// The size of the view in GUI units, rounded down.
    pub fn size(&self) -> Option<Size> {
        let mut out = AmmlSize { width: 0, height: 0 };
        // SAFETY: as for `kind`.
        #[allow(unsafe_code)]
        let status = unsafe { (self.table.view_size)(self.view.0, &mut out) };
        status.is_ok().then(|| Size::from(out))
    }

    /// The rectangle of the panel that holds the slots of the screen, in GUI units. `None` for a screen or UI
    /// profile with no known panel, and while the panel is hidden.
    pub fn content_rect(&self) -> Option<Rect> {
        let mut out = AmmlRect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };
        // SAFETY: as for `kind`.
        #[allow(unsafe_code)]
        let status = unsafe { (self.table.view_content_rect)(self.view.0, &mut out) };
        status.is_ok().then(|| Rect::from(out))
    }

    /// The last pointer position of the view, in GUI units.
    pub fn pointer(&self) -> Option<Point> {
        let mut out = AmmlPoint { x: 0.0, y: 0.0 };
        // SAFETY: as for `kind`.
        #[allow(unsafe_code)]
        let status = unsafe { (self.table.view_pointer)(self.view.0, &mut out) };
        status.is_ok().then(|| Point::from(out))
    }

    pub fn input_mode(&self) -> Option<InputMode> {
        let mut out = AmmlInputMode::UNKNOWN;
        // SAFETY: as for `kind`.
        #[allow(unsafe_code)]
        let status = unsafe { (self.table.view_input_mode)(self.view.0, &mut out) };
        status.is_ok().then(|| InputMode::from(out))
    }

    /// The container screen of the view, for its slots, its cursor and its moves. `None` when the screen is not a
    /// container screen, and for a loader whose `ui` table has no container calls. A loader that does not serve
    /// `Area::UI_SCREEN.at(2)` can still give `Some`: each read of the screen then answers `None` and each action
    /// `NotAvailable`.
    pub fn container_screen(&self) -> Option<ContainerScreen<'f>> {
        self.table.container_slot_count?;
        let mut screen = 0u64;
        // SAFETY: as for `kind`; `screen` is a live local.
        #[allow(unsafe_code)]
        let status = unsafe { (self.table.view_container)(self.view.0, &mut screen) };
        (status.is_ok() && screen != 0)
            .then(|| ContainerScreen::new(self.table, self.inventory, self.module, self.view, ScreenId(screen)))
    }

    /// The UI profile option of the game.
    pub fn ui_profile(&self) -> Option<UiProfile> {
        let mut out = AmmlUiProfile::CLASSIC;
        // SAFETY: as for `kind`.
        #[allow(unsafe_code)]
        let status = unsafe { (self.table.view_ui_profile)(self.view.0, &mut out) };
        status.is_ok().then(|| UiProfile::from(out))
    }
}
