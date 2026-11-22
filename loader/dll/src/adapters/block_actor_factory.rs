use crate::{InventoryAdapter, LoaderState};

/// `createBlockActor(BlockPos const&, BlockType const&)`, 26.63: rcx the 16-byte result, rdx the position, r8 the type.
type CreateFn = unsafe extern "C" fn(*mut [usize; 2], *const [i32; 3], usize) -> *mut [usize; 2];
/// `createBlockActor(BlockPos const&, bool, BlockType const&)`, 70.26: rcx the result, rdx the position, r8b the flag,
/// r9 the type.
type CreateWithFlagFn = unsafe extern "C" fn(*mut [usize; 2], *const [i32; 3], u8, usize) -> *mut [usize; 2];
/// The level's flag slot (`Level.isClientSide`): rcx the level; the byte in al.
type FlagFn = unsafe extern "C" fn(usize) -> u8;

/// `VanillaBlockActorFactory::createBlockActor` in the form of the running build (survey section 7, row E1): the 26.63
/// form takes the position and the block type; the 70.26 form takes a flag byte between them, which the loader asks the
/// level for with the slot `BlockActor::loadStatic` calls before its own factory call (`mov rax,[r9]; mov rax,[rax+9E8h];
/// call`, 0x72cf900, entry `Level.isClientSide`), as the game does. The form is the entry that resolved: each name is
/// gated to its own build.
pub struct BlockActorFactory {
    create: usize,
    flag_slot: Option<usize>,
}

impl BlockActorFactory {
    /// The factory of the running build; `None` when neither form resolved, or the 70.26 form without its flag slot.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let number = |name: &str| {
            state.resolve(name).ok().and_then(|value| usize::try_from(value).ok()).filter(|value| *value != 0)
        };
        if let Some(create) = number("VanillaBlockActorFactory.createBlockActor") {
            return Some(Self { create, flag_slot: None });
        }
        let create = number("VanillaBlockActorFactory.createBlockActorWithFlag")?;
        Some(Self { create, flag_slot: Some(number("Level.isClientSide")?) })
    }

    /// The `shared_ptr<BlockActor>` (actor, control block) the factory makes for the block type at `block_type` at
    /// `pos`, on the level at `level` (read only by the 70.26 form); empty for a type with no block entity.
    ///
    /// # Safety
    /// `block_type` is a live block type and `level` the live level of the local player; game thread.
    pub unsafe fn make(&self, block_type: usize, pos: [i32; 3], level: usize) -> Option<[usize; 2]> {
        let mut out = [0usize; 2];
        match self.flag_slot {
            None => {
                // SAFETY: the entry is the 26.63 factory, whose signature is `CreateFn`.
                let create = unsafe { std::mem::transmute::<usize, CreateFn>(self.create) };
                // SAFETY: a 16-byte result, a position that lives for the call and a live block type.
                unsafe { create(&mut out, &pos, block_type) };
            }
            Some(slot) => {
                let function = InventoryAdapter::virtual_fn(level, slot)?;
                // SAFETY: the flag slot of the level vtable holds a member function that takes only `this`.
                let flag_of = unsafe { std::mem::transmute::<usize, FlagFn>(function) };
                // SAFETY: called on the live level; its body reads one byte of the level.
                let flag = unsafe { flag_of(level) };
                // SAFETY: the entry is the 70.26 factory, whose signature is `CreateWithFlagFn`.
                let create = unsafe { std::mem::transmute::<usize, CreateWithFlagFn>(self.create) };
                // SAFETY: as above, with the flag the level gave.
                unsafe { create(&mut out, &pos, flag, block_type) };
            }
        }
        (out[0] != 0).then_some(out)
    }
}
