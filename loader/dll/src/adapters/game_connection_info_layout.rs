use crate::{GameConnectionInfoLayout2663, GameConnectionInfoLayout2670, GameConnectionRead};

/// The layout of the game's `Social::GameConnectionInfo` that a build has, chosen by the value of the derived
/// `GameConnectionInfo.size` entry. Any other size has no layout the loader knows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameConnectionInfoLayout {
    /// 26.63: 2F0h bytes, host at 08h, port at 8Ch.
    Size2663,
    /// 70.26: 308h bytes, host at 20h, port at A4h.
    Size2670,
}

impl GameConnectionInfoLayout {
    /// The layout of a value of `size` bytes; `None` for a size no known layout has.
    pub fn for_size(size: usize) -> Option<Self> {
        match size {
            0x2F0 => Some(Self::Size2663),
            0x308 => Some(Self::Size2670),
            _ => None,
        }
    }

    /// The size of the value; the engaged flag of a `std::optional` of it is the byte at this offset.
    pub fn size(self) -> usize {
        match self {
            Self::Size2663 => 0x2F0,
            Self::Size2670 => 0x308,
        }
    }

    /// Reads the head of the value at `info`; the address need not be aligned.
    ///
    /// # Safety
    /// `info` is the address of a constructed `Social::GameConnectionInfo` of this layout, readable for `size` bytes.
    pub unsafe fn read(self, info: usize) -> GameConnectionRead {
        match self {
            Self::Size2663 => {
                // SAFETY: the caller guarantees a constructed value of this layout; the mirror is its head, and the
                // copy is a plain bitwise one (the host text stays in the game's memory, which the value keeps).
                let head =
                    unsafe { std::ptr::with_exposed_provenance::<GameConnectionInfoLayout2663>(info).read_unaligned() };
                GameConnectionRead {
                    connection_type: head.connection_type,
                    host: head.host.to_str().ok().map(str::to_string),
                    port: head.port,
                }
            }
            Self::Size2670 => {
                // SAFETY: the caller guarantees a constructed value of this layout; the mirror is its head, and the
                // copy is a plain bitwise one (the host text stays in the game's memory, which the value keeps).
                let head =
                    unsafe { std::ptr::with_exposed_provenance::<GameConnectionInfoLayout2670>(info).read_unaligned() };
                GameConnectionRead {
                    connection_type: head.connection_type,
                    host: head.host.to_str().ok().map(str::to_string),
                    port: head.port,
                }
            }
        }
    }
}
