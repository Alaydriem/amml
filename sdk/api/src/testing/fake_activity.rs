/// What `Client::note_activity` gives on a `FakeGame`: `Ok` (the call is recorded in `client_writes` as
/// `note_activity`), `Refused` (the game is not ready) or `NotAvailable` (no adapter on this build).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FakeActivity {
    #[default]
    Ok,
    Refused,
    NotAvailable,
}
