use egui::{Response, Ui};

use crate::state::InteractionState;

/// A setting row's own control: drawn by the caller into the row's right side, with the row's state.
pub type CustomControl<'a> = Box<dyn FnOnce(&mut Ui, InteractionState) -> Response + 'a>;
