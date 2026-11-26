use amml_abi::AmmlActorBox;

use crate::Aabb;

/// One actor that `BlockSource::actors_in` found: a copy of its box, and whether it is the local player.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActorBox {
    pub aabb: Aabb,
    pub is_local_player: bool,
}

impl From<AmmlActorBox> for ActorBox {
    fn from(actor: AmmlActorBox) -> Self {
        Self { aabb: actor.aabb.into(), is_local_player: actor.is_local_player != 0 }
    }
}

impl From<ActorBox> for AmmlActorBox {
    fn from(actor: ActorBox) -> Self {
        Self { aabb: actor.aabb.into(), is_local_player: u8::from(actor.is_local_player), _reserved: [0; 3] }
    }
}
