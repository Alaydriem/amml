use std::time::Instant;

use amml_api::{ButtonHold, InputLease, KeyHold};

use crate::CheckPhase;

/// The state of one run of the in-world checks: the phase, its tick count and the leases and holds
/// the checks own. Dropping it releases every hold and ends both leases.
pub struct CheckRun {
    pub phase: CheckPhase,
    /// Client ticks since the phase began, or since its action was sent.
    pub ticks: u32,
    /// Whether the phase sent its action.
    pub sent: bool,
    /// The shared lease without blocks that injects the keys, buttons, wheel, turn and move.
    pub lease: Option<InputLease>,
    /// The shared `FULL` lease of the gamepad phase.
    pub full: Option<InputLease>,
    pub key: Option<KeyHold>,
    /// When the W hold went down.
    pub key_since: Option<Instant>,
    pub button: Option<ButtonHold>,
    /// The last top screen the title phase logged (`Start`, `Other`, `Pause`, or `none` for a refused read).
    pub screen_seen: Option<&'static str>,
    /// The turn steps sent in the turn phase.
    pub turn_steps: u32,
    /// When the turn phase sent its first step, and its last.
    pub turn_started: Option<Instant>,
    pub turn_ended: Option<Instant>,
}

impl Default for CheckRun {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckRun {
    pub const fn new() -> Self {
        Self {
            phase: CheckPhase::Title,
            ticks: 0,
            sent: false,
            lease: None,
            full: None,
            key: None,
            key_since: None,
            button: None,
            screen_seen: None,
            turn_steps: 0,
            turn_started: None,
            turn_ended: None,
        }
    }

    /// Moves to the next phase.
    pub fn advance(&mut self) {
        self.phase = self.phase.next();
        self.ticks = 0;
        self.sent = false;
    }
}
