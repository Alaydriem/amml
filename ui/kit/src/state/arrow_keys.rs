use egui::{EventFilter, Id, Ui};

/// A widget that reads the arrow keys while it has the focus. egui moves the focus to the next
/// widget on a bare arrow key unless the focused widget asks to keep it, so the key would act on
/// two widgets at once.
pub(crate) struct ArrowKeys;

impl ArrowKeys {
    /// Keeps the arrow keys on `id` while it has the focus. Call it on every pass: egui drops the
    /// request when the focus moves.
    pub fn keep(ui: &Ui, id: Id) {
        let filter = EventFilter { horizontal_arrows: true, vertical_arrows: true, ..EventFilter::default() };
        ui.memory_mut(|memory| memory.set_focus_lock_filter(id, filter));
    }
}
