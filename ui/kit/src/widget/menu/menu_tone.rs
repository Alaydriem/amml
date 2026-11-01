/// A menu item's tone: Danger colours its icon and label `--color-danger`; its fills do not change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MenuTone {
    #[default]
    Normal,
    Danger,
}
