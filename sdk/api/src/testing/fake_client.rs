use crate::testing::{FakeActivity, FakeWorld};
use crate::{
    Aabb, CameraKind, GameType, HitResult, PermissionLevel, Rotation, Screen, StartWorldOutcome, Vec3, WorldInfo,
};

/// The client a `FakeGame` serves after `set_client`: what the `Client` reads give. `mouse_captured`
/// is `None` until the game grabbed or released the mouse once, as in the loader. `pause_on_focus_lost` is
/// the game option `pause_menu_on_focus_lost`: with it off, `open_pause_screen` gives `Refused`.
#[derive(Clone, Debug, PartialEq)]
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
    /// What `Client::in_gameplay` gives (needs `Area::CLIENT_IN_GAMEPLAY`).
    pub in_gameplay: bool,
    /// Whether the client shows the title screen, for `Client::on_title_screen` (needs `Area::CLIENT_TITLE_SCREEN`);
    /// `false` while `local_player` is set, as in the loader.
    pub title_screen: bool,
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
    /// What `LocalPlayer::head_position` gives.
    pub head_position: Option<Vec3>,
    /// What `LocalPlayer::health` gives.
    pub health: Option<i32>,
    /// What `LocalPlayer::view_vector` gives.
    pub view_vector: Option<Vec3>,
    /// What `LocalPlayer::aabb` gives.
    pub aabb: Option<Aabb>,
    /// What `LocalPlayer::game_type` gives.
    pub game_type: Option<GameType>,
    /// What `LocalPlayer::permission_level` gives.
    pub permission_level: Option<PermissionLevel>,
    /// What `Dimension::sky_darken` gives; needs `dimension`.
    pub sky_darken: Option<u8>,
    /// The world behind `LocalPlayer::block_source`: `Some` gives a block source, `None` gives none.
    pub world: Option<FakeWorld>,
    /// What `Client::world_info` gives; `None` is the loader's refusal (no world joined).
    pub world_info: Option<WorldInfo>,
    /// What `CameraControl::active_camera` gives; `None` is a game with no active camera.
    pub active_camera: Option<CameraKind>,
    /// What `GameMode::build_delay` gives (needs `Area::PLAYER_BUILD`).
    pub build_delay: Option<u32>,
    /// What `GameMode::max_pick_range` gives (needs `Area::PLAYER_PICK_RANGE`).
    pub max_pick_range: Option<f32>,
    /// What `LocalPlayer::hit_result` gives (needs `Area::PLAYER_HIT_RESULT`).
    pub hit_result: Option<HitResult>,
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
            in_gameplay: false,
            title_screen: false,
            start_world_answer: Some(StartWorldOutcome::Started),
            activity: FakeActivity::Ok,
            position: None,
            eye_position: None,
            rotation: None,
            dimension: None,
            head_position: None,
            health: None,
            view_vector: None,
            aabb: None,
            game_type: None,
            permission_level: None,
            sky_darken: None,
            world: None,
            world_info: None,
            active_camera: None,
            build_delay: None,
            max_pick_range: None,
            hit_result: None,
        }
    }
}
