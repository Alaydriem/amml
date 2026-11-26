use amml_runtime::SavedWaypoint;

use crate::LoaderState;

/// Where the waypoints of the local player's locator bar are and what one waypoint holds, from the entries
/// `ClientInstance.getLocalPlayer`, `LocalPlayer.mClientLocatorBar`, `ClientLocatorBar.listHead`,
/// `ClientLocatorBar.nodeWaypoint`, `ClientWaypoint.mColor`, `ClientWaypoint.mColorEngaged`, `ClientWaypoint.mVisible`,
/// `ClientEntityWaypoint.mActorId` and `ClientEntityWaypoint.vtable`.
///
/// The waypoints are the values of a `std::list` of `unique_ptr<ClientWaypoint>` nodes that sits in an
/// `unordered_map`; `list_head` is the offset of the pointer to the sentinel node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocatorBarLayout {
    /// The vtable slot of `LocalPlayer* IClientInstance::getLocalPlayer()`.
    pub local_player_slot: usize,
    /// `mClientLocatorBar` inside `LocalPlayer`.
    pub bar: usize,
    /// The pointer to the list sentinel inside `ClientLocatorBar`.
    pub list_head: usize,
    /// The `unique_ptr<ClientWaypoint>` inside one list node.
    pub node_waypoint: usize,
    /// `mColor` (an `optional<mce::Color>`) inside `ClientWaypoint`.
    pub color: usize,
    /// The engaged byte of `mColor`.
    pub color_engaged: usize,
    /// The visible flag inside `ClientWaypoint`.
    pub visible: usize,
    /// The `ActorUniqueID` inside `ClientEntityWaypoint`.
    pub actor_id: usize,
    /// The address of the `ClientEntityWaypoint` vtable.
    pub entity_vtable: usize,
}

impl LocatorBarLayout {
    /// The offset of the next pointer inside a list node. No entry gives it: it is the first member of an MSVC
    /// `std::list` node. The first walk logs it with the number of nodes it found.
    pub const NODE_NEXT: usize = 0;
    /// The most nodes one walk visits; it guards against a corrupt list.
    pub const MAX_NODES: usize = 1024;

    /// The layout of the resolved entries; `None` when one is missing or the colour members do not fit together.
    pub fn from_state(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let layout = Self {
            local_player_slot: value("ClientInstance.getLocalPlayer")?,
            bar: value("LocalPlayer.mClientLocatorBar")?,
            list_head: value("ClientLocatorBar.listHead")?,
            node_waypoint: value("ClientLocatorBar.nodeWaypoint")?,
            color: value("ClientWaypoint.mColor")?,
            color_engaged: value("ClientWaypoint.mColorEngaged")?,
            visible: value("ClientWaypoint.mVisible")?,
            actor_id: value("ClientEntityWaypoint.mActorId")?,
            entity_vtable: value("ClientEntityWaypoint.vtable").filter(|address| *address != 0)?,
        };
        layout.fits().then_some(layout)
    }

    /// Whether the engaged byte lies inside the colour bytes a frame saves, because a restore of those bytes then
    /// leaves no tag behind, and whether the colour and the bar offsets do not overflow.
    pub fn fits(&self) -> bool {
        let saved_end = self.color.checked_add(SavedWaypoint::COLOR_BYTES);
        saved_end.is_some_and(|end| self.color_engaged >= self.color && self.color_engaged < end)
            && self.bar.checked_add(self.list_head).is_some()
    }
}
