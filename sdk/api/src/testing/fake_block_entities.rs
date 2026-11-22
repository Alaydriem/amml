use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use amml_abi::{AmmlBlockEntityGhost, AmmlBlockHandle, AmmlGhostEntityOptions, AmmlStatus};
use amml_runtime::{GhostEntity, GhostEntityBook, GhostEntityKey, GhostEntityRefusal};

/// The block entity ghosts of a `FakeGame`, with the loader's checks: a chest pair one block along x or z only, on a
/// block whose kind is `chest`, and only with its area; a skull pose with a face up to 5 and a turn up to 15, on a block
/// whose kind is `skull`, and only with its area; 256 live ghosts at most; a draw only inside the running
/// `BlockEntityPass` dispatch; a ghost of an earlier world is `Stale`. Each accepted call adds a line: `create
/// kind=<kind> nbt=<len>` (with ` pair=x,y,z` for a chest pair and ` skull=face,turn` for a skull pose) and
/// `draw <id> x,y,z`.
#[derive(Default)]
pub struct FakeBlockEntities {
    book: GhostEntityBook,
    kinds: RefCell<BTreeMap<u64, &'static str>>,
    calls: RefCell<Vec<String>>,
    next_frame: Cell<u64>,
    running: Cell<u64>,
}

impl FakeBlockEntities {
    // The owner of the one fake mod.
    const OWNER: usize = 1;
    // The highest face the game numbers and the highest skull turn, in sixteenths.
    const MAX_FACE: u8 = 5;
    const MAX_TURN: u8 = 15;

    /// The accepted calls so far, one line each.
    pub fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }

    /// The number of live ghosts of the fake mod.
    pub fn live(&self) -> usize {
        self.book.len()
    }

    pub(crate) fn set_kind(&self, block: AmmlBlockHandle, kind: &'static str) {
        self.kinds.borrow_mut().insert(block.0, kind);
    }

    /// A pass dispatch starts; gives its frame id.
    pub(crate) fn enter(&self) -> u64 {
        let frame = self.next_frame.get() + 1;
        self.next_frame.set(frame);
        self.running.set(frame);
        frame
    }

    pub(crate) fn leave(&self) {
        self.running.set(0);
    }

    pub(crate) fn world_left(&self) {
        self.book.world_left();
        let _ = self.book.take_due();
    }

    /// A `create` of the loader after its area, gate and world checks. `chest_pair` and `skull` are whether their areas
    /// are offered.
    pub(crate) fn create(
        &self,
        block: AmmlBlockHandle,
        nbt_len: usize,
        options: &AmmlGhostEntityOptions,
        chest_pair: bool,
        skull: bool,
    ) -> Result<AmmlBlockEntityGhost, AmmlStatus> {
        let kind = self.kinds.borrow().get(&block.0).copied().unwrap_or("block");
        let mut line = format!("create kind={kind} nbt={nbt_len}");
        if options.has_chest_pair == 1 {
            let pair = options.chest_pair;
            if !matches!((pair.x, pair.y, pair.z), (-1 | 1, 0, 0) | (0, 0, -1 | 1)) {
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            if !chest_pair {
                return Err(AmmlStatus::NOT_AVAILABLE);
            }
            if kind != "chest" {
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            line.push_str(&format!(" pair={},{},{}", pair.x, pair.y, pair.z));
        }
        let pose = if options.has_skull == 1 {
            let pose = options.skull;
            if pose.face.0 > Self::MAX_FACE || pose.turn > Self::MAX_TURN {
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            if !skull {
                return Err(AmmlStatus::NOT_AVAILABLE);
            }
            if kind != "skull" {
                return Err(AmmlStatus::INVALID_ARGUMENT);
            }
            line.push_str(&format!(" skull={},{}", pose.face.0, pose.turn));
            Some((pose.face.0, pose.turn))
        } else {
            None
        };
        let address = usize::try_from(block.0).map_err(|_| AmmlStatus::FAILED)?;
        let ghost = GhostEntity { actor: address, control: 0, block: address, skull: pose, chest_paired: false };
        let key = self.book.create(Self::OWNER, ghost).map_err(Self::refusal)?;
        self.calls.borrow_mut().push(line);
        Ok(AmmlBlockEntityGhost { id: key.id, generation: key.generation })
    }

    pub(crate) fn free(&self, ghost: AmmlBlockEntityGhost) -> Result<(), AmmlStatus> {
        self.book.free(Self::OWNER, Self::key(ghost)).map_err(Self::refusal)?;
        let _ = self.book.take_due();
        Ok(())
    }

    pub(crate) fn draw(&self, frame: u64, ghost: AmmlBlockEntityGhost, pos: [i32; 3]) -> Result<(), AmmlStatus> {
        if frame == 0 || frame != self.running.get() {
            return Err(AmmlStatus::REFUSED);
        }
        self.book.entry(Self::OWNER, Self::key(ghost)).map_err(Self::refusal)?;
        self.calls.borrow_mut().push(format!("draw {} {},{},{}", ghost.id, pos[0], pos[1], pos[2]));
        Ok(())
    }

    fn key(ghost: AmmlBlockEntityGhost) -> GhostEntityKey {
        GhostEntityKey { id: ghost.id, generation: ghost.generation }
    }

    fn refusal(refusal: GhostEntityRefusal) -> AmmlStatus {
        match refusal {
            GhostEntityRefusal::Stale => AmmlStatus::STALE,
            GhostEntityRefusal::NotOwner | GhostEntityRefusal::TooMany => AmmlStatus::REFUSED,
        }
    }
}
