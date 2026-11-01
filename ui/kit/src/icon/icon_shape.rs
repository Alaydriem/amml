use super::DrawnIcon;

/// What an icon is made of: a Segoe Fluent Icons code point, or a painted shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconShape {
    Font(char),
    Drawn(DrawnIcon),
}
