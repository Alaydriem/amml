use std::collections::HashMap;

use crate::{ShapeKind, ShapeRefusal, ShapeRequest, ShapeSend};

/// The shapes the mods drew in the game through the loader, and the game ids they have.
///
/// A mod's shape ids are its own: the book maps each `(mod, id)` to a game network id from its own counter, which
/// starts at `ID_BASE` and never gives an id twice, so two mods never move each other's shapes. A batch of more than
/// `MAX_BATCH` requests is refused whole. A remove of an id the book does not hold sends nothing. A draw of a live id
/// with another kind sends the remove of the old game id and the draw of a new one, because the game cannot change the
/// kind of a shape object. The shapes belong to the client packet handler of one generation: a batch, or a disable, of
/// another generation finds them dropped by the game and forgets them. A mod's disable gives the game ids of its live
/// shapes to remove; a world leave forgets every shape. The book makes no game call.
#[derive(Debug, Default)]
pub struct ShapeBook {
    live: HashMap<(usize, u64), (u64, ShapeKind)>,
    next: u64,
    generation: u32,
}

impl ShapeBook {
    /// The most requests one batch takes.
    pub const MAX_BATCH: usize = 800;
    /// The first game network id the book gives ("AMML" in the high half), away from the small ids a server's
    /// script shapes use.
    pub const ID_BASE: u64 = 0x414D_4D4C_0000_0000;

    pub fn new() -> Self {
        Self { live: HashMap::new(), next: 0, generation: 0 }
    }

    /// The payloads that carry the `requests` of the mod `module` to the game, in order, for the handler of
    /// `generation`. `TooMany` (and no change) for more than `MAX_BATCH` requests.
    pub fn apply(
        &mut self,
        generation: u32,
        module: usize,
        requests: &[ShapeRequest],
    ) -> Result<Vec<ShapeSend>, ShapeRefusal> {
        if requests.len() > Self::MAX_BATCH {
            return Err(ShapeRefusal::TooMany { count: requests.len(), max: Self::MAX_BATCH });
        }
        self.follow(generation);
        let mut sends = Vec::with_capacity(requests.len());
        for (op, request) in requests.iter().enumerate() {
            let key = (module, request.id);
            let held = self.live.get(&key).copied();
            match (request.kind, held) {
                (None, None) => {}
                (None, Some((network_id, _))) => {
                    self.live.remove(&key);
                    sends.push(ShapeSend { op, network_id, kind: None });
                }
                (Some(kind), Some((network_id, held_kind))) if held_kind == kind => {
                    sends.push(ShapeSend { op, network_id, kind: Some(kind) });
                }
                (Some(kind), held) => {
                    if let Some((old, _)) = held {
                        sends.push(ShapeSend { op, network_id: old, kind: None });
                    }
                    let network_id = self.next_id();
                    self.live.insert(key, (network_id, kind));
                    sends.push(ShapeSend { op, network_id, kind: Some(kind) });
                }
            }
        }
        Ok(sends)
    }

    /// The mod `module` is disabled: its shapes are forgotten. Gives the game ids of its live shapes to remove when
    /// the handler is still of `generation`; none when the game dropped them with an older handler.
    pub fn disable(&mut self, module: usize, generation: u32) -> Vec<u64> {
        let same = generation == self.generation;
        let mut removed: Vec<u64> = Vec::new();
        self.live.retain(|(owner, _), (network_id, _)| {
            if *owner != module {
                return true;
            }
            if same {
                removed.push(*network_id);
            }
            false
        });
        removed.sort_unstable();
        removed
    }

    /// Before `WorldLeft`: every shape is forgotten; the game drops them with the world.
    pub fn world_left(&mut self) {
        self.live.clear();
    }

    /// The number of live shapes of the mod `module`.
    pub fn live(&self, module: usize) -> usize {
        self.live.keys().filter(|(owner, _)| *owner == module).count()
    }

    // A batch for the handler of `generation`: the shapes of another generation were dropped by the game.
    fn follow(&mut self, generation: u32) {
        if generation != self.generation {
            self.live.clear();
            self.generation = generation;
        }
    }

    fn next_id(&mut self) -> u64 {
        let id = Self::ID_BASE.wrapping_add(self.next);
        self.next = self.next.wrapping_add(1);
        id
    }
}
