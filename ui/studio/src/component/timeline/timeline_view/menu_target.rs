/// What a right-click on the timeline opens a menu for. The menus themselves are the host's (Plan 08).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuTarget {
    /// The ruler, at a time in seconds ("New scene here").
    Ruler(f64),
    Scene(usize),
    Lane(usize),
    Key {
        lane: usize,
        key: usize,
    },
}
