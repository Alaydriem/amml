use egui::Ui;

/// A page header's trailing parts: drawn by the caller into the bar's right side, right to left.
pub type PageHeaderTrailing<'a> = Box<dyn FnOnce(&mut Ui) + 'a>;
