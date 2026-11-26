/// The game's `ActorType` of an actor: the kind in the low byte (`0x20` zombie, `0x3F` player) and family bits above it
/// (`0x100` mob, `0x800` monster). 0 when the loader found no type for the actor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ActorTypeId(pub u32);
