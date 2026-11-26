use std::hint::black_box;
use std::sync::atomic::{AtomicBool, Ordering};

use amml_data::{EntryKind, Resolved};

use super::{FakeVtable, InfoGame, SessionGame};

static IN_GAME: AtomicBool = AtomicBool::new(false);
// A stand-in `MinecraftGame` vtable: slot 1 is `isInGame`.
static GAME_VTABLE: FakeVtable<2> = FakeVtable([JoinGame::unused as *const (), JoinGame::is_in_game as *const ()]);

/// Stand-ins and builders for the session facts of a join: the `std::string` arguments of `onStartJoinGame`, a
/// by-value connection info and join context of `startExternalNetworkWorld` and `joinMultiplayerWithAddress`, and a
/// `MinecraftGame` with `isInGame`.
pub struct JoinGame;

impl JoinGame {
    /// The session entries, the world info entries (the storage ones too) for a connection info of `size` bytes, and the
    /// join entries.
    pub fn entries(size: usize) -> Vec<(&'static str, Resolved)> {
        let mut entries = SessionGame::entries(&[]);
        entries.extend(InfoGame::entries(size));
        let (url, name) = if size == InfoGame::SIZE_2670 { (0x40, 0x80) } else { (0x28, 0x68) };
        let field = |value| Resolved { kind: EntryKind::Field, value, recipes_agreeing: 2 };
        entries.push((
            "ClientInstance.startExternalNetworkWorld",
            Resolved { kind: EntryKind::VSlot, value: 3, recipes_agreeing: 2 },
        ));
        entries.push(("GameConnectionInfo.mUnresolvedUrl", field(url)));
        entries.push(("GameConnectionInfo.mExternalServerName", field(name)));
        entries.push(("MinecraftGame.isInGame", Resolved { kind: EntryKind::VSlot, value: 1, recipes_agreeing: 2 }));
        entries.push((
            "MinecraftGame.joinMultiplayerWithAddress",
            Resolved { kind: EntryKind::VSlot, value: 3, recipes_agreeing: 2 },
        ));
        entries
    }

    /// The address of a leaked MSVC `std::string` holding `text`: inline up to 15 bytes, else a leaked heap block.
    pub fn string(text: &str) -> usize {
        let block: &'static mut [u64] = Box::leak(vec![0u64; 4].into_boxed_slice());
        Self::write_string(block.as_mut_ptr() as usize, text.as_bytes());
        block.as_ptr() as usize
    }

    /// The address of a leaked connection info of `size` bytes (0x2F0 or 0x308) with its host, unresolved URL, external
    /// server name and port set.
    pub fn info(size: usize, host: &str, url: &str, name: &str, port: i32) -> usize {
        let block: &'static mut [u64] = Box::leak(vec![0u64; size.div_ceil(8)].into_boxed_slice());
        let at = block.as_mut_ptr() as usize;
        let (host_at, url_at, name_at, port_at) =
            if size == InfoGame::SIZE_2670 { (0x20, 0x40, 0x80, 0xA4) } else { (0x08, 0x28, 0x68, 0x8C) };
        Self::write_string(at + host_at, host.as_bytes());
        Self::write_string(at + url_at, url.as_bytes());
        Self::write_string(at + name_at, name.as_bytes());
        // SAFETY: `at + port_at` is inside the leaked block, which is larger than the offset.
        unsafe { std::ptr::with_exposed_provenance_mut::<i32>(at + port_at).write_unaligned(port) };
        at
    }

    /// `info` with its connection type (the u16 at +0, on both layouts) set to `kind`; 8 is `UnknownIP`.
    pub fn typed(info: usize, kind: u16) -> usize {
        // SAFETY: `info` is a leaked block from `info`, larger than two bytes.
        unsafe { std::ptr::with_exposed_provenance_mut::<u16>(info).write_unaligned(kind) };
        info
    }

    /// The address of a leaked join context of 0xB0 bytes whose server transfer byte (22h) is `transfer`.
    pub fn context(transfer: bool) -> usize {
        let block: &'static mut [u64] = Box::leak(vec![0u64; 0xB0 / 8].into_boxed_slice());
        let at = block.as_mut_ptr() as usize;
        // SAFETY: `at + 0x22` is inside the leaked block.
        unsafe { std::ptr::with_exposed_provenance_mut::<u8>(at + 0x22).write(u8::from(transfer)) };
        at
    }

    /// The address of a stand-in `MinecraftGame` whose `isInGame` gives `value` from now on.
    pub fn game(value: bool) -> usize {
        IN_GAME.store(value, Ordering::SeqCst);
        Self::game_object()
    }

    pub fn set_in_game(value: bool) {
        IN_GAME.store(value, Ordering::SeqCst);
    }

    fn game_object() -> usize {
        static OBJECT: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        *OBJECT.get_or_init(|| {
            let block: &'static mut [u64] = Box::leak(vec![0u64; 2].into_boxed_slice());
            block[0] = std::ptr::from_ref(&GAME_VTABLE) as usize as u64;
            block.as_ptr() as usize
        })
    }

    #[inline(never)]
    extern "C" fn unused(this: usize) {
        black_box(this);
    }

    #[inline(never)]
    extern "C" fn is_in_game(this: usize) -> u8 {
        black_box(this);
        u8::from(IN_GAME.load(Ordering::SeqCst))
    }

    // Writes the 32 bytes of an MSVC `std::string` holding `bytes` at `at`.
    fn write_string(at: usize, bytes: &[u8]) {
        let mut words = [0u64; 4];
        if bytes.len() <= 15 {
            let mut inline = [0u8; 16];
            inline[..bytes.len()].copy_from_slice(bytes);
            words[0] = u64::from_le_bytes(inline[..8].try_into().unwrap());
            words[1] = u64::from_le_bytes(inline[8..].try_into().unwrap());
            words[3] = 15;
        } else {
            let block: &'static mut [u8] = Box::leak(bytes.to_vec().into_boxed_slice());
            words[0] = block.as_ptr() as usize as u64;
            words[3] = bytes.len() as u64;
        }
        words[2] = bytes.len() as u64;
        // SAFETY: `at` is inside a leaked block with room for the 32 bytes; the write is unaligned-safe.
        unsafe { std::ptr::with_exposed_provenance_mut::<[u64; 4]>(at).write_unaligned(words) };
    }
}
