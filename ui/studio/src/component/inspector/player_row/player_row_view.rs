use amml_i18n::text::Text;

/// What a row of the Players tab shows. Every word comes from the host.
pub struct PlayerRowView {
    /// The letter on the avatar tile.
    pub initial: Text,
    pub name: Text,
    /// How far the player is from the camera.
    pub distance: Text,
    /// The follow button's words: "Follow", or "Following" while the camera follows this player.
    pub follow: Text,
    pub go_to: Text,
    /// The camera follows this player now.
    pub following: bool,
}
