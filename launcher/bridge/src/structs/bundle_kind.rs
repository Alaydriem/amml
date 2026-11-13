// The installer bundle a launcher build came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleKind {
    Nsis,
    Msi,
}
