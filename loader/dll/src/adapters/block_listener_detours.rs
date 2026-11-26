use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{BlockListener, Boundary};

/// `void BlockSourceListener::onBlockChanged(BlockSource&, BlockPos const&, uint layer, Block const& block, Block const&
/// oldBlock, int updateFlags, ActorBlockSyncMessage const*, BlockChangedEventTarget, Actor*)`: rcx the listener, rdx the
/// block source, r8 the position, r9d the layer, then on the stack the new block, the old block, the flags, the sync
/// message, the event target (a byte) and the source actor.
type ChangedFn = unsafe extern "C" fn(usize, usize, *const [i32; 3], u32, usize, usize, i32, usize, u8, usize);
/// `void BlockSourceListener::onSourceDestroyed(BlockSource&)`: rcx the listener, rdx the block source.
type DestroyedFn = unsafe extern "C" fn(usize, usize);

// The address of the listener object the loader has attached; 0 while none is.
static OBJECT: AtomicUsize = AtomicUsize::new(0);

/// The two functions that take the place of `onSourceDestroyed` and `onBlockChanged` in the vtable of the loader's
/// block listener object. They are never hooks on the game's code: only the loader's own object points to them.
///
/// With no object attached, `onBlockChanged` does one relaxed load and returns. A panic inside the adapter stops at
/// the thunk.
pub struct BlockListenerDetours;

impl BlockListenerDetours {
    /// The address of the `onBlockChanged` thunk.
    pub fn changed_address() -> usize {
        Self::on_block_changed as ChangedFn as usize
    }

    /// The address of the `onSourceDestroyed` thunk.
    pub fn destroyed_address() -> usize {
        Self::on_source_destroyed as DestroyedFn as usize
    }

    /// Sets the object the thunks answer for; 0 ends it.
    pub(crate) fn set_object(address: usize) {
        OBJECT.store(address, Ordering::Release);
    }

    // The arguments past the position are the game's own and are not read.
    #[allow(clippy::too_many_arguments)]
    unsafe extern "C" fn on_block_changed(
        this: usize,
        source: usize,
        pos: *const [i32; 3],
        layer: u32,
        new_block: usize,
        old_block: usize,
        _flags: i32,
        _sync: usize,
        _target: u8,
        _actor: usize,
    ) {
        if this == 0 || this != OBJECT.load(Ordering::Relaxed) || pos.is_null() {
            return;
        }
        // SAFETY: the game passes a live `BlockPos` of three ints, valid for the call.
        let pos = unsafe { pos.read_unaligned() };
        Boundary::run((), || {
            BlockListener::with((), |adapter| adapter.changed(source, pos, layer, new_block, old_block));
        });
    }

    unsafe extern "C" fn on_source_destroyed(this: usize, source: usize) {
        if this == 0 || this != OBJECT.load(Ordering::Acquire) {
            return;
        }
        Boundary::run((), || BlockListener::with((), |adapter| adapter.source_destroyed(source)));
    }
}
