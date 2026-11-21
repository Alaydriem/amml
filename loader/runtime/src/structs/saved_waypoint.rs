use crate::WaypointTags;

/// One waypoint of the locator bar that a frame tagged: where it is, the exact bytes of its colour, and the actor it
/// follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SavedWaypoint {
    waypoint: usize,
    // The 20 bytes of the `optional<mce::Color>`: four floats, the engaged byte and its padding.
    color: [u8; Self::COLOR_BYTES],
    actor: i64,
    has_actor: bool,
}

impl SavedWaypoint {
    /// The size of the saved colour: four floats, the engaged byte and three bytes of padding.
    pub const COLOR_BYTES: usize = 20;
    // The engaged byte of the optional.
    const ENGAGED: usize = 16;

    pub fn new(waypoint: usize, color: [u8; Self::COLOR_BYTES], actor: Option<i64>) -> Self {
        Self { waypoint, color, actor: actor.unwrap_or(0), has_actor: actor.is_some() }
    }

    /// The address of the waypoint.
    pub fn waypoint(&self) -> usize {
        self.waypoint
    }

    /// The colour bytes the waypoint had before it was tagged.
    pub fn color_bytes(&self) -> &[u8; Self::COLOR_BYTES] {
        &self.color
    }

    /// The actor the waypoint follows; `None` for a waypoint of another kind.
    pub fn actor(&self) -> Option<i64> {
        self.has_actor.then_some(self.actor)
    }

    /// The colour the game would have given to the icon blit: the saved colour packed as `0xAARRGGBB`, or white
    /// when the colour was empty.
    pub fn vanilla_packed(&self) -> u32 {
        if self.color[Self::ENGAGED] == 0 {
            return 0xFFFF_FFFF;
        }
        let lane = |index: usize| {
            f32::from_le_bytes([
                self.color[index * 4],
                self.color[index * 4 + 1],
                self.color[index * 4 + 2],
                self.color[index * 4 + 3],
            ])
        };
        WaypointTags::pack([lane(0), lane(1), lane(2), lane(3)])
    }
}
