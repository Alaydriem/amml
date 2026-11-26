use crate::{SavedWaypoint, WaypointWrite};

/// The waypoints of the locator bar that one render pass tagged. The loader writes a tag (the index of the waypoint in
/// the list) in place of the colour of each visible waypoint before the game draws the bar, reads the tag back in the
/// icon blit to find the waypoint, and puts every colour back when the pass ends.
///
/// The memory reads and writes are the caller's: `tag` and `restore` only say what to write where.
pub struct WaypointTags {
    saved: Vec<SavedWaypoint>,
    active: bool,
}

impl WaypointTags {
    /// The most waypoints one frame tags; a frame with more is left untagged.
    pub const CAPACITY: usize = 256;
    // The first lane of every tag.
    const MARKER: u8 = 0xA5;

    pub fn new() -> Self {
        Self { saved: Vec::with_capacity(Self::CAPACITY), active: false }
    }

    /// The colour that marks a waypoint with `index`: the red lane is the marker, green and blue hold the index, and
    /// alpha is 1. Each lane packs to its exact byte.
    pub fn encode(index: u16) -> [f32; 4] {
        [Self::lane(Self::MARKER), Self::lane((index >> 8) as u8), Self::lane((index & 0xFF) as u8), 1.0]
    }

    /// The index in a colour packed as `0xAARRGGBB`; `None` when the colour is not a tag.
    pub fn decode(packed: u32) -> Option<u16> {
        let alpha = (packed >> 24) as u8;
        let red = (packed >> 16) as u8;
        if alpha != 0xFF || red != Self::MARKER {
            return None;
        }
        Some((packed & 0xFFFF) as u16)
    }

    /// The game's packing of a colour: each lane clamped to 0..=1, scaled by 255 and truncated, as `0xAARRGGBB`.
    pub fn pack(color: [f32; 4]) -> u32 {
        (Self::byte(color[3]) << 24) | (Self::byte(color[0]) << 16) | (Self::byte(color[1]) << 8) | Self::byte(color[2])
    }

    // A float that packs back to exactly `value` under truncation.
    fn lane(value: u8) -> f32 {
        let exact = f32::from(value) / 255.0;
        if Self::byte(exact) == u32::from(value) { exact } else { (f32::from(value) + 0.5) / 255.0 }
    }

    fn byte(lane: f32) -> u32 {
        (lane.clamp(0.0, 1.0) * 255.0) as u32
    }

    /// Whether a frame is tagged now.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Starts a frame: keeps `waypoints` in order and calls `write(address, WaypointWrite::Tag(tag))` for each, with the
    /// tag colour of its index. A frame still open is restored first, with `WaypointWrite::Saved`. False, with no tag
    /// written and no frame, when more than `CAPACITY` waypoints come.
    pub fn tag(
        &mut self,
        waypoints: impl IntoIterator<Item = SavedWaypoint>,
        mut write: impl FnMut(usize, WaypointWrite),
    ) -> bool {
        self.restore(|waypoint, bytes| write(waypoint, WaypointWrite::Saved(*bytes)));
        for waypoint in waypoints {
            if self.saved.len() == Self::CAPACITY {
                self.saved.clear();
                return false;
            }
            self.saved.push(waypoint);
        }
        for (index, waypoint) in self.saved.iter().enumerate() {
            write(waypoint.waypoint(), WaypointWrite::Tag(Self::encode(index as u16)));
        }
        self.active = !self.saved.is_empty();
        self.active
    }

    /// The waypoint of the tag in `packed`; `None` outside a tagged frame, for a colour that is not a tag and for an
    /// index the frame does not hold.
    pub fn lookup(&self, packed: u32) -> Option<&SavedWaypoint> {
        if !self.active {
            return None;
        }
        self.saved.get(usize::from(Self::decode(packed)?))
    }

    /// Ends the frame: calls `write(address, bytes)` with the saved colour bytes of every waypoint, and empties the
    /// frame.
    pub fn restore(&mut self, mut write: impl FnMut(usize, &[u8; SavedWaypoint::COLOR_BYTES])) {
        for waypoint in &self.saved {
            write(waypoint.waypoint(), waypoint.color_bytes());
        }
        self.saved.clear();
        self.active = false;
    }
}

impl Default for WaypointTags {
    fn default() -> Self {
        Self::new()
    }
}
