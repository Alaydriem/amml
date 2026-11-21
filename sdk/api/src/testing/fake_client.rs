use crate::testing::FakeActivity;
use crate::{Rotation, Screen, StartWorldOutcome, Vec3};

/// The client a `FakeGame` serves after `set_client`: what the `Client` reads give. `mouse_captured`
/// is `None` until the game grabbed or released the mouse once, as in the loader. `pause_on_focus_lost` is
/// the game option `pause_menu_on_focus_lost`: with it off, `open_pause_screen` gives `Refused`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FakeClient {
    pub local_player: bool,
    pub world_active: bool,
    pub showing_menu: bool,
    pub top_screen: Screen,
    pub hide_hud: bool,
    pub mouse_captured: Option<bool>,
    pub pause_on_focus_lost: bool,
    /// What `LocalPlayer::is_alive` gives for the local player; `None` is a read the loader cannot make.
    pub player_alive: Option<bool>,
    /// What `Client::can_join` gives.
    pub can_join: bool,
    /// What `Client::start_local_world` gives; `None` is the game's "not ready" answer (`NotReady`).
    pub start_world_answer: Option<StartWorldOutcome>,
    /// What `Client::note_activity` gives.
    pub activity: FakeActivity,
    /// What `LocalPlayer::position` (the feet) gives; `None` is a read the loader cannot make.
    pub position: Option<Vec3>,
    /// What `LocalPlayer::eye_position` gives.
    pub eye_position: Option<Vec3>,
    /// What `LocalPlayer::rotation` gives.
    pub rotation: Option<Rotation>,
    /// The id `LocalPlayer::dimension` then `Dimension::id` gives; `None` gives no dimension.
    pub dimension: Option<i32>,
}

impl Default for FakeClient {
    /// The title screen: no player, no world, no menu flag, the HUD shown, the mouse not yet seen and the
    /// pause-on-focus-lost option on, no alive state, ready for a join, a world start that the game takes and an any-input call that the game runs.
    fn default() -> Self {
        Self {
            local_player: false,
            world_active: false,
            showing_menu: false,
            top_screen: Screen::Start,
            hide_hud: false,
            mouse_captured: None,
            pause_on_focus_lost: true,
            player_alive: None,
            can_join: true,
            start_world_answer: Some(StartWorldOutcome::Started),
            activity: FakeActivity::Ok,
            position: None,
            eye_position: None,
            rotation: None,
            dimension: None,
        }
    }
}
