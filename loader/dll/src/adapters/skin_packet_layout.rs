use amml_abi::AmmlStatus;

use crate::{LoaderState, ObjectRead};

/// Where the uuid and the skin of a `PlayerSkinPacket` are, from the entries `PlayerSkinPacket.mUUID` and `.mSkin`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkinPacketLayout {
    pub uuid: usize,
    pub skin: usize,
}

impl SkinPacketLayout {
    /// The layout of the resolved entries; `None` when one is missing.
    pub fn from_state(state: &LoaderState) -> Option<Self> {
        let field = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        Some(Self { uuid: field("PlayerSkinPacket.mUUID")?, skin: field("PlayerSkinPacket.mSkin")? })
    }

    /// The uuid of the packet.
    ///
    /// # Safety
    /// `packet` is the live `PlayerSkinPacket` the game gave its handler.
    pub unsafe fn uuid(&self, packet: usize) -> Result<[u8; 16], AmmlStatus> {
        // SAFETY: the caller guarantees a live packet that holds the 16-byte uuid at `uuid`.
        unsafe { ObjectRead::value::<[u8; 16]>(packet, self.uuid) }.ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    /// The address of the `SerializedSkinRef` of the packet.
    pub fn skin_ref(&self, packet: usize) -> Option<usize> {
        packet.checked_add(self.skin)
    }
}
