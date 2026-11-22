use std::sync::atomic::{AtomicBool, Ordering};

use amml_runtime::ProtocolValues;

use crate::ObjectRead;

static OPEN_LOGGED: AtomicBool = AtomicBool::new(false);
static CLOSE_LOGGED: AtomicBool = AtomicBool::new(false);
static RESPONSE_LOGGED: AtomicBool = AtomicBool::new(false);
static INTERACT_LOGGED: AtomicBool = AtomicBool::new(false);
static CLOSE_WRITE_LOGGED: AtomicBool = AtomicBool::new(false);

/// The hand-written mirrors of the packets the request path reads and writes, as offsets from the packet object.
/// They are not derived entries: a packet is a game object whose payload starts after the 0x30-byte `Packet` base.
///
/// Each offset is marked with its source.
/// - Exe-proven on both gate builds (read from the executable with `inspect packets` and `inspect fields`): the sizes,
///   `ContainerClosePacket` `mContainerType` (+31h) and `mServerInitiatedClose` (+32h), the `ItemStackResponsePacket`
///   vector at +30h with the stride 0x30, and of an element the result at +0, the request id at +10h and the containers
///   vector at +18h; of a container info (stride 0x28) the name at +0 and the slots vector at +10h; of a slot info
///   (stride 0x58) the slot at +1 and the net id at +4 (survey section 7, row I6).
/// - Not proven by the exe (they come from the mirror of the old hotbar-manager, which ran on 26.62): the
///   `ContainerOpenPacket` id (+30h) and type (+31h), the `ContainerClosePacket` id (+30h), the `InteractPacket`
///   action (+30h) and target (+38h). The first value of each read is
///   logged once, so a wrong offset shows in the log.
pub struct PacketLayout;

impl PacketLayout {
    pub const INTERACT_SIZE: usize = 0x58;
    pub const CONTAINER_OPEN_SIZE: usize = 0x50;
    pub const CONTAINER_CLOSE_SIZE: usize = 0x38;
    pub const RESPONSE_SIZE: usize = 0x50;

    // Not proven by the exe.
    pub const INTERACT_ACTION_AT: usize = 0x30;
    pub const INTERACT_TARGET_AT: usize = 0x38;
    pub const OPEN_ID_AT: usize = 0x30;
    pub const OPEN_TYPE_AT: usize = 0x31;
    pub const CLOSE_ID_AT: usize = 0x30;
    // Proven by the exe.
    pub const RESPONSE_RESULT_AT: usize = 0x00;
    pub const CLOSE_TYPE_AT: usize = 0x31;
    pub const CLOSE_SERVER_AT: usize = 0x32;
    pub const RESPONSES_AT: usize = 0x30;
    pub const RESPONSE_STRIDE: usize = 0x30;
    pub const RESPONSE_ID_AT: usize = 0x10;
    /// More answers than any packet of one client carries; a larger count is a misread.
    pub const MAX_ANSWERS: usize = 256;
    // The `ItemStackResponseInfo` (0x30 bytes) holds its `mContainers` vector at +18h; a container info (0x28 bytes) its
    // name at +0 and its `mSlots` vector at +10h; a slot info (0x58 bytes) its slot at +1 and its net id at +4. All
    // proven by the exe on both gate builds (survey section 7, row I6).
    pub const RESPONSE_CONTAINERS_AT: usize = 0x18;
    pub const CONTAINER_STRIDE: usize = 0x28;
    pub const CONTAINER_NAME_AT: usize = 0x00;
    pub const CONTAINER_SLOTS_AT: usize = 0x10;
    pub const SLOT_STRIDE: usize = 0x58;
    pub const SLOT_AT: usize = 0x01;
    pub const SLOT_NET_ID_AT: usize = 0x04;
    /// More containers or slots in one answer than a request can name; a larger count is a misread.
    pub const MAX_NAMED: usize = 64;

    /// The container id and type of a `ContainerOpenPacket`.
    ///
    /// # Safety
    /// `packet` is the live `ContainerOpenPacket` the game gave its handler.
    pub(crate) unsafe fn container_open(packet: usize) -> Option<(i8, i8)> {
        // SAFETY: the caller guarantees a live packet of `CONTAINER_OPEN_SIZE` bytes; both offsets are inside it.
        let id = unsafe { ObjectRead::value::<i8>(packet, Self::OPEN_ID_AT) }?;
        // SAFETY: as above.
        let kind = unsafe { ObjectRead::value::<i8>(packet, Self::OPEN_TYPE_AT) }?;
        if !OPEN_LOGGED.swap(true, Ordering::AcqRel) {
            curia::info!("first container open read", { container_id: i64::from(id), container_type: i64::from(kind) });
        }
        Some((id, kind))
    }

    /// The container id of a `ContainerClosePacket`.
    ///
    /// # Safety
    /// `packet` is the live `ContainerClosePacket` the game gave its handler.
    pub(crate) unsafe fn container_close(packet: usize) -> Option<i8> {
        // SAFETY: the caller guarantees a live packet of `CONTAINER_CLOSE_SIZE` bytes; the offset is inside it.
        let id = unsafe { ObjectRead::value::<i8>(packet, Self::CLOSE_ID_AT) }?;
        if !CLOSE_LOGGED.swap(true, Ordering::AcqRel) {
            curia::info!("first container close read", { container_id: i64::from(id) });
        }
        Some(id)
    }

    /// The (wire request id, accepted) of each answer of an `ItemStackResponsePacket`; empty for a vector that does not
    /// look sane.
    ///
    /// # Safety
    /// `packet` is the live `ItemStackResponsePacket` the game gave its handler.
    pub(crate) unsafe fn responses(packet: usize) -> Vec<(i32, bool)> {
        // SAFETY: the caller guarantees a live packet; the vector (begin, end) sits at `RESPONSES_AT`.
        let Some([begin, end]) = (unsafe { ObjectRead::value::<[usize; 2]>(packet, Self::RESPONSES_AT) }) else {
            return Vec::new();
        };
        let Some(bytes) = end.checked_sub(begin) else { return Vec::new() };
        if begin == 0 || bytes % Self::RESPONSE_STRIDE != 0 || bytes / Self::RESPONSE_STRIDE > Self::MAX_ANSWERS {
            if bytes != 0 {
                curia::warn!("item stack response not read", { bytes: bytes });
            }
            return Vec::new();
        }
        let mut answers = Vec::with_capacity(bytes / Self::RESPONSE_STRIDE);
        for index in 0..bytes / Self::RESPONSE_STRIDE {
            let element = begin + index * Self::RESPONSE_STRIDE;
            // SAFETY: an element of the live answer vector; both offsets are inside its 0x30 bytes.
            let result = unsafe { ObjectRead::value::<u8>(element, Self::RESPONSE_RESULT_AT) };
            // SAFETY: as above.
            let id = unsafe { ObjectRead::value::<i32>(element, Self::RESPONSE_ID_AT) };
            if let (Some(result), Some(id)) = (result, id) {
                if !RESPONSE_LOGGED.swap(true, Ordering::AcqRel) {
                    curia::info!("first item stack response read", { request_id: i64::from(id), result: result });
                }
                answers.push((id, result == ProtocolValues::RESULT_SUCCESS));
            }
        }
        answers
    }

    /// The (wire request id, slot, net id) of every slot of the player's hotbar container that the answers of an
    /// `ItemStackResponsePacket` name; empty for a vector that does not read.
    ///
    /// # Safety
    /// `packet` is the live `ItemStackResponsePacket` the game gave its handler.
    pub(crate) unsafe fn hotbar_net_ids(packet: usize) -> Vec<(i32, u8, i32)> {
        // SAFETY: the caller guarantees a live packet; the vector (begin, end) sits at `RESPONSES_AT`.
        let Some([begin, end]) = (unsafe { ObjectRead::value::<[usize; 2]>(packet, Self::RESPONSES_AT) }) else {
            return Vec::new();
        };
        let mut named = Vec::new();
        let Some(bytes) = end.checked_sub(begin) else { return named };
        if begin == 0 || bytes % Self::RESPONSE_STRIDE != 0 || bytes / Self::RESPONSE_STRIDE > Self::MAX_ANSWERS {
            return named;
        }
        for index in 0..bytes / Self::RESPONSE_STRIDE {
            let element = begin + index * Self::RESPONSE_STRIDE;
            // SAFETY: an element of the live answer vector; the offsets are inside its 0x30 bytes.
            let id = unsafe { ObjectRead::value::<i32>(element, Self::RESPONSE_ID_AT) };
            // SAFETY: as above.
            let containers = unsafe { ObjectRead::value::<[usize; 2]>(element, Self::RESPONSE_CONTAINERS_AT) };
            let (Some(id), Some([first, last])) = (id, containers) else { continue };
            for (name, slots) in Self::containers(first, last) {
                if name != ProtocolValues::HOTBAR {
                    continue;
                }
                for (slot, net_id) in Self::slots(slots.0, slots.1) {
                    named.push((id, slot, net_id));
                }
            }
        }
        named
    }

    // The (name, slots vector) of each container info in `first..last`; none for a vector that does not read.
    fn containers(first: usize, last: usize) -> Vec<(u8, (usize, usize))> {
        let Some(bytes) = last.checked_sub(first) else { return Vec::new() };
        if first == 0 || bytes % Self::CONTAINER_STRIDE != 0 || bytes / Self::CONTAINER_STRIDE > Self::MAX_NAMED {
            return Vec::new();
        }
        (0..bytes / Self::CONTAINER_STRIDE)
            .filter_map(|index| {
                let container = first + index * Self::CONTAINER_STRIDE;
                // SAFETY: a container info of the live answer; both offsets are inside its 0x28 bytes.
                let name = unsafe { ObjectRead::value::<u8>(container, Self::CONTAINER_NAME_AT) }?;
                // SAFETY: as above.
                let [begin, end] = unsafe { ObjectRead::value::<[usize; 2]>(container, Self::CONTAINER_SLOTS_AT) }?;
                Some((name, (begin, end)))
            })
            .collect()
    }

    // The (slot, net id) of each slot info in `first..last`; none for a vector that does not read.
    fn slots(first: usize, last: usize) -> Vec<(u8, i32)> {
        let Some(bytes) = last.checked_sub(first) else { return Vec::new() };
        if first == 0 || bytes % Self::SLOT_STRIDE != 0 || bytes / Self::SLOT_STRIDE > Self::MAX_NAMED {
            return Vec::new();
        }
        (0..bytes / Self::SLOT_STRIDE)
            .filter_map(|index| {
                let info = first + index * Self::SLOT_STRIDE;
                // SAFETY: a slot info of the live answer; both offsets are inside its 0x58 bytes.
                let slot = unsafe { ObjectRead::value::<u8>(info, Self::SLOT_AT) }?;
                // SAFETY: as above.
                let net_id = unsafe { ObjectRead::value::<i32>(info, Self::SLOT_NET_ID_AT) }?;
                Some((slot, net_id))
            })
            .collect()
    }

    /// Writes the action and the target of an `InteractPacket` the game made.
    ///
    /// # Safety
    /// `packet` is a live `InteractPacket` made by the game's factory for its id, `INTERACT_SIZE` bytes.
    pub(crate) unsafe fn write_interact(packet: usize, action: u8, target: u64) {
        // SAFETY: the caller guarantees a live packet of `INTERACT_SIZE` bytes; both offsets are inside it.
        unsafe {
            std::ptr::with_exposed_provenance_mut::<u8>(packet + Self::INTERACT_ACTION_AT).write(action);
            std::ptr::with_exposed_provenance_mut::<u64>(packet + Self::INTERACT_TARGET_AT).write_unaligned(target);
        }
        if !INTERACT_LOGGED.swap(true, Ordering::AcqRel) {
            curia::info!("first interact packet written", {
                action: action,
                target: target,
                action_at: Self::INTERACT_ACTION_AT,
                target_at: Self::INTERACT_TARGET_AT
            });
        }
    }

    /// Writes the id, the type (none) and the not-server-initiated flag of a `ContainerClosePacket` the game made.
    ///
    /// # Safety
    /// `packet` is a live `ContainerClosePacket` made by the game's factory for its id, `CONTAINER_CLOSE_SIZE`
    /// bytes.
    pub(crate) unsafe fn write_close(packet: usize, container_id: i8) {
        // SAFETY: the caller guarantees a live packet of `CONTAINER_CLOSE_SIZE` bytes; the three offsets are inside it.
        unsafe {
            std::ptr::with_exposed_provenance_mut::<i8>(packet + Self::CLOSE_ID_AT).write(container_id);
            std::ptr::with_exposed_provenance_mut::<i8>(packet + Self::CLOSE_TYPE_AT).write(ProtocolValues::NO_TYPE);
            std::ptr::with_exposed_provenance_mut::<u8>(packet + Self::CLOSE_SERVER_AT).write(0);
        }
        if !CLOSE_WRITE_LOGGED.swap(true, Ordering::AcqRel) {
            curia::info!("first container close written", {
                container_id: i64::from(container_id),
                id_at: Self::CLOSE_ID_AT
            });
        }
    }
}
