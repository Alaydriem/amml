use amml_abi::ScreenRender;

use crate::lent_scope::LentScope;
use crate::{ContainerScreen, ScreenId, ScreenView, UiDraw, ViewId};

/// Typed reads of a `ScreenRender`.
pub trait ScreenRenderExt {
    /// The screen of the view that was drawn.
    fn screen(&self) -> ScreenId;

    /// The id of the view that was drawn. The loader refuses it once this call of the listener is over.
    fn view_id(&self) -> ViewId;

    /// The number of client ticks raised so far.
    fn tick(&self) -> u64;

    /// The view, for reads: `None` when the loader has no ui table, and for any `ScreenRender` that is not the
    /// event of a running listener call on this thread (a copy, or one kept past its call).
    fn view(&self) -> Option<ScreenView<'_>>;

    /// The container screen of the view (`ScreenView::container_screen`), under the same rule as `view`.
    fn container_screen(&self) -> Option<ContainerScreen<'_>>;

    /// The draw calls of the view, under the same rule as `view`. Every call is refused after the listener
    /// returns, and while the area `ui.draw` is not offered.
    fn draw(&self) -> Option<UiDraw<'_>>;
}

impl ScreenRenderExt for ScreenRender {
    fn screen(&self) -> ScreenId {
        ScreenId(self.screen)
    }

    fn view_id(&self) -> ViewId {
        ViewId(self.view)
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn view(&self) -> Option<ScreenView<'_>> {
        let lent = LentScope::find_view(std::ptr::from_ref(self))?;
        Some(ScreenView::new(lent.table, lent.inventory, lent.module, self.view_id()))
    }

    fn container_screen(&self) -> Option<ContainerScreen<'_>> {
        self.view()?.container_screen()
    }

    fn draw(&self) -> Option<UiDraw<'_>> {
        let table = LentScope::find(std::ptr::from_ref(self))?;
        Some(UiDraw::new(table, self.view_id()))
    }
}
