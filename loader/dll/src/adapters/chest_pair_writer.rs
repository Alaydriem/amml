use crate::{InventoryAdapter, LoaderState};

/// `bool BlockActor::isType(BlockActorType) const`: rcx the actor, dl the type; the bool in al.
type IsTypeFn = unsafe extern "C" fn(usize, u8) -> u8;

/// Makes a chest block actor the lead of a large chest (area `render.chest_pair`): the game's own chest test, then the
/// pair members the chest renderer reads.
///
/// Exe readings (26.63 / 70.26): the chest renderer (0x6a6ad80 / 0x8a43140) asks `isType` with `mov dl,2` before it
/// reads a chest member (0x6a6ae09 / 0x8a431cd), so 2 is the chest type; it compares `mLargeChestPaired` with 0 only
/// (`cmp qword ptr [r14+308h],0` / `[r14+310h]`, six sites on each build); it takes the pair axis from bit 1 of the pair
/// flags (`movzx eax,byte ptr [r14+2F8h]; and al,2; shr al,1` at 0x6a6b27e, `[r14+300h]` at 0x8a4355f) and compares
/// `mLargeChestPairedPosition[axis * 2]` with `mPosition[axis * 2]` (`mov ecx,[r14+rax*8+310h]; cmp ecx,[r14+rax*8+8]`
/// / `+318h`), so the axis picks x or z. `ChestBlockActor::pairWith` sets that bit when the partner has the same x
/// (70.26 0x72e6309: `mov ecx,[r15+8]; cmp ecx,[rsi+8]; sete cl; add cl,cl; and al,0FDh; or al,cl`), so bit 1 means a
/// pair along z.
pub struct ChestPairWriter {
    is_type_slot: usize,
    position_at: usize,
    flags_at: usize,
    paired_at: usize,
    paired_position_at: usize,
}

impl ChestPairWriter {
    /// `BlockActorType::Chest`: every chest-family actor answers `isType(2)`.
    pub const CHEST_TYPE: u8 = 2;
    /// Bit 1 of the pair flags: the pair runs along z.
    pub const ALONG_Z: u8 = 0b10;

    /// The writer of the running build; `None` when an entry of the area is missing.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        Some(Self {
            is_type_slot: value("BlockActor.isType")?,
            position_at: value("BlockActor.mPosition")?,
            flags_at: value("ChestBlockActor.mPairFlags")?,
            paired_at: value("ChestBlockActor.mLargeChestPaired")?,
            paired_position_at: value("ChestBlockActor.mLargeChestPairedPosition")?,
        })
    }

    /// Whether the game's own `isType(Chest)` says the actor at `actor` is a `ChestBlockActor`.
    ///
    /// # Safety
    /// `actor` is a live block actor the loader holds a use of; game thread.
    pub unsafe fn is_chest(&self, actor: usize) -> bool {
        let Some(function) = InventoryAdapter::virtual_fn(actor, self.is_type_slot) else { return false };
        // SAFETY: the slot of `BlockActor.isType` in the vtable of a live actor holds `isType`.
        let is_type = unsafe { std::mem::transmute::<usize, IsTypeFn>(function) };
        // SAFETY: called on its own live actor with a type byte; it reads the actor only.
        unsafe { is_type(actor, Self::CHEST_TYPE) != 0 }
    }

    /// Clears the partner pointer of the chest at `actor` when it names the actor itself (as `write` left it), so the
    /// game's destructor never follows a pointer to the object it destroys. Called before the loader's last use of a
    /// chest it paired is released.
    ///
    /// # Safety
    /// `actor` is a live `ChestBlockActor` that `write` paired, of which the loader still holds a use.
    pub unsafe fn unpair(&self, actor: usize) {
        // SAFETY: the caller guarantees a live ChestBlockActor; the pointer at the derived offset is inside it.
        unsafe {
            let paired = std::ptr::with_exposed_provenance_mut::<usize>(actor + self.paired_at);
            if paired.read_unaligned() == actor {
                paired.write_unaligned(0);
            }
        }
    }

    /// Makes the chest at `actor` the lead of a large chest whose other half is at `offset` from it (one block along x
    /// or z). The partner pointer is the actor's own address (the renderer only compares it with null, and it lives as
    /// long as the actor), the partner position is the actor's `mPosition` moved by `offset`, and bit 1 of the flags is
    /// set for a pair along z and cleared for one along x; the other flag bits stay.
    ///
    /// # Safety
    /// `actor` is a live `ChestBlockActor` (`is_chest` true) the loader holds a use of; game thread.
    pub unsafe fn write(&self, actor: usize, offset: [i32; 3]) {
        // SAFETY: `mPosition` of a live actor.
        let own = unsafe { std::ptr::with_exposed_provenance::<[i32; 3]>(actor + self.position_at).read_unaligned() };
        let partner = [own[0].wrapping_add(offset[0]), own[1].wrapping_add(offset[1]), own[2].wrapping_add(offset[2])];
        // SAFETY: the caller guarantees a live ChestBlockActor; the flag byte, the pointer and the position at the
        // derived offsets are inside it (their recipes check 2A0h..380h, the size of the actor is 380h on 26.63).
        unsafe {
            let flags = std::ptr::with_exposed_provenance_mut::<u8>(actor + self.flags_at);
            let old = flags.read();
            flags.write(if offset[0] == 0 { old | Self::ALONG_Z } else { old & !Self::ALONG_Z });
            std::ptr::with_exposed_provenance_mut::<usize>(actor + self.paired_at).write_unaligned(actor);
            std::ptr::with_exposed_provenance_mut::<[i32; 3]>(actor + self.paired_position_at).write_unaligned(partner);
        }
    }
}
