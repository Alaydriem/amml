/// One column of a table: its share of the width inside the row's side insets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableColumn {
    pub width_fraction: f32,
}
