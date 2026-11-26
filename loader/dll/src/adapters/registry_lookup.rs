use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};

use amml_abi::AmmlStatus;
use amml_msvc::{MsvcHeap, MsvcString};

use crate::{ClientAdapter, HashedStringLayout30, HashedStringLayout38, LoaderState, NameReader};

/// `_lookupByNameImpl(HashedString const& name, int data, LookupByNameImplResolve, bool logNotFound) const`: rcx the
/// registry, rdx the 16-byte result {BlockType*, Block*}, r8 the name, r9d the data, then the resolve and log arguments.
type LookupFn = unsafe extern "C" fn(usize, *mut [usize; 2], *const c_void, i32, i32, u8) -> *mut [usize; 2];

/// The block state for data 0 of a block type found by name in the game's block registry
/// (`BlockTypeRegistry.instance` + 18h, `BlockTypeRegistry.lookupByName`), with a `HashedString` of the build's layout
/// (`HashedString.layout`) whose hash is `HashedString::computeHash` (FNV-1). The text is made on the game heap and
/// freed after the call. Game thread only.
///
/// The lookup runs with the `Block` resolve (1): only then does the game fill result+8, with the state its legacy data
/// function gives for data 0, which starts from the type's default state (`BlockType+240h`). With resolve 0 the game
/// gives the type and a null block on 26.63 and 70.26.
pub struct RegistryLookup {
    state: &'static LoaderState,
    registry: usize,
    lookup: LookupFn,
    layout: u64,
    no_state_logged: AtomicBool,
}

impl RegistryLookup {
    // `BlockTypeRegistry.instance` is the Owner global; the registry it owns is 18h after it.
    const REGISTRY_OFFSET: usize = 0x18;
    // `LookupByNameImplResolve`: 1 makes the game fill the block of the result, 0 gives the type only.
    const RESOLVE_BLOCK: i32 = 1;
    // The legacy data value of the state the lookup gives.
    const DATA: i32 = 0;

    /// The lookup of this build; `None` when an entry is missing or the `HashedString` layout is not known.
    pub fn new(state: &'static LoaderState) -> Option<Self> {
        let number = |name: &str| {
            state.resolve(name).ok().and_then(|value| usize::try_from(value).ok()).filter(|value| *value != 0)
        };
        let layout = state.resolve("HashedString.layout").ok()?;
        NameReader::new(layout)?;
        let lookup = number("BlockTypeRegistry.lookupByName")?;
        Some(Self {
            state,
            registry: number("BlockTypeRegistry.instance")?.checked_add(Self::REGISTRY_OFFSET)?,
            // SAFETY: the address is the derived body of `_lookupByNameImpl`, whose signature is `LookupFn`.
            lookup: unsafe { std::mem::transmute::<usize, LookupFn>(lookup) },
            layout,
            no_state_logged: AtomicBool::new(false),
        })
    }

    /// The state for data 0 of the block type `name` (the default state for a type whose data 0 is its default);
    /// `None` for a name the registry does not know, and for a type the game gives no state for (logged once).
    /// `NOT_AVAILABLE` without the game heap.
    pub fn data_zero_state(&self, name: &str) -> Result<Option<usize>, AmmlStatus> {
        if !self.state.heap().is_available() {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        let heap = MsvcHeap::new(ClientAdapter::heap_new, ClientAdapter::heap_delete);
        let hash = Self::hash(name.as_bytes());
        // SAFETY: the game heap is available (checked above).
        let text = unsafe { MsvcString::new_in(&heap, name) }.map_err(|_| AmmlStatus::FAILED)?;
        let mut result = [0usize; 2];
        if self.layout == NameReader::LAYOUT_30 {
            let mut name = HashedStringLayout30::new(hash, text);
            // SAFETY: the registry and a HashedString of the build's layout that lives for the call; data 0, the
            // `Block` resolve and no log.
            unsafe {
                (self.lookup)(
                    self.registry,
                    &mut result,
                    std::ptr::from_ref(&name).cast(),
                    Self::DATA,
                    Self::RESOLVE_BLOCK,
                    0,
                )
            };
            // SAFETY: the text was made on this heap above and is not used after.
            unsafe { name.text.release(&heap) };
        } else {
            let mut name = HashedStringLayout38::with_string(hash, text);
            // SAFETY: as above, with the 38h layout.
            unsafe {
                (self.lookup)(
                    self.registry,
                    &mut result,
                    std::ptr::from_ref(&name).cast(),
                    Self::DATA,
                    Self::RESOLVE_BLOCK,
                    0,
                )
            };
            if let Some(text) = name.string_mut() {
                // SAFETY: as above.
                unsafe { text.release(&heap) };
            }
        }
        let [block_type, block] = result;
        if block_type != 0 && block == 0 && !self.no_state_logged.swap(true, Ordering::AcqRel) {
            curia::warn!("registry lookup found the block type but no block state", { name: name });
        }
        Ok((block_type != 0 && block != 0).then_some(block))
    }

    /// `HashedString::computeHash`: FNV-1 over the bytes from the offset basis; 0 for an empty text.
    pub fn hash(text: &[u8]) -> u64 {
        const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
        const PRIME: u64 = 0x0000_0100_0000_01b3;
        if text.is_empty() {
            return 0;
        }
        text.iter().fold(OFFSET_BASIS, |hash, byte| hash.wrapping_mul(PRIME) ^ u64::from(*byte))
    }
}
