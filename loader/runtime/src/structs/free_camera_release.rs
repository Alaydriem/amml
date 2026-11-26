/// What the loader does when a mod is disabled: `clear` sends the clear instruction to the captured handler,
/// `drop_packet` gives the kept camera instruction packet back to the game.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FreeCameraRelease {
    pub clear: bool,
    pub drop_packet: bool,
}
