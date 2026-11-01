/// What the Rust listener saw, written by its callbacks.
#[derive(Default)]
pub struct Record {
    pub name: Option<String>,
    pub destroyed_with_delete_flag: bool,
    pub state_dropped: bool,
}
