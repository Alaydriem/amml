// What a mod install did to the instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModChange {
    Installed,
    // the version that was there before, when it had one
    Updated { from: Option<String> },
}
