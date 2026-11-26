use amml_abi::AmmlActorEntry;

use crate::{Aabb, ActorTypeId};

/// One actor that `BlockSource::actors_in_entries` found: a copy of its box, its `ActorType` and whether it is the
/// local player.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActorEntry {
    pub aabb: Aabb,
    pub actor_type: ActorTypeId,
    pub is_local_player: bool,
}

impl ActorEntry {
    pub fn new(aabb: Aabb, actor_type: ActorTypeId, is_local_player: bool) -> Self {
        Self { aabb, actor_type, is_local_player }
    }
}

impl From<AmmlActorEntry> for ActorEntry {
    fn from(entry: AmmlActorEntry) -> Self {
        Self {
            aabb: entry.aabb.into(),
            actor_type: ActorTypeId(entry.actor_type),
            is_local_player: entry.is_local_player != 0,
        }
    }
}

impl From<ActorEntry> for AmmlActorEntry {
    fn from(entry: ActorEntry) -> Self {
        Self {
            aabb: entry.aabb.into(),
            actor_type: entry.actor_type.0,
            is_local_player: u8::from(entry.is_local_player),
            _reserved: [0; 3],
        }
    }
}
