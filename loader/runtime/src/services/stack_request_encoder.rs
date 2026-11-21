use crate::{RequestAction, SlotRef};

/// Writes the payload of item stack requests the way the game's `BinaryStream` writers do, so that `Packet::read` of
/// an `ItemStackRequestPacket` reads it: the number of requests, then for each its id, its actions, the strings to
/// filter (none) and their origin (-1). Each action starts with its index in the game's action variant, then its
/// type. The `93 01` packet header is not written: the loader fills a packet the game made from the payload.
pub struct StackRequestEncoder;

impl StackRequestEncoder {
    const ACTION_PLACE: u8 = 1;
    const ACTION_SWAP: u8 = 2;
    const ACTION_DROP: u8 = 3;
    const NOT_RANDOMLY: u8 = 0;
    const ORIGIN_NONE: i32 = -1;

    /// Several requests in one payload, in order, each with its id and its one action. The server answers each by
    /// its id.
    pub fn batch(requests: &[(i32, RequestAction)]) -> Vec<u8> {
        let mut out = Vec::with_capacity(4 + requests.len() * 28);
        Self::write_unsigned_var_int(&mut out, u32::try_from(requests.len()).unwrap_or(u32::MAX));
        for (request_id, action) in requests {
            Self::write_var_int(&mut out, *request_id);
            Self::write_unsigned_var_int(&mut out, 1);
            Self::write_action(&mut out, *action);
            Self::write_unsigned_var_int(&mut out, 0);
            out.extend_from_slice(&Self::ORIGIN_NONE.to_le_bytes());
        }
        out
    }

    fn write_action(out: &mut Vec<u8>, action: RequestAction) {
        match action {
            RequestAction::Swap { from, to } => {
                Self::write_kind(out, Self::ACTION_SWAP);
                Self::write_slot(out, from);
                Self::write_slot(out, to);
            }
            RequestAction::Place { count, from, to } => {
                Self::write_place(out, count, from, SlotRef { net_id: 0, ..to });
            }
            RequestAction::PlaceOnto { count, from, to } => Self::write_place(out, count, from, to),
            RequestAction::Drop { count, from } => {
                Self::write_kind(out, Self::ACTION_DROP);
                out.push(count);
                Self::write_slot(out, from);
                out.push(Self::NOT_RANDOMLY);
            }
        }
    }

    fn write_place(out: &mut Vec<u8>, count: u8, from: SlotRef, to: SlotRef) {
        Self::write_kind(out, Self::ACTION_PLACE);
        out.push(count);
        Self::write_slot(out, from);
        Self::write_slot(out, to);
    }

    // `BinaryStream::writeVarInt`: the zigzag form of `value`, as an unsigned varint.
    fn write_var_int(out: &mut Vec<u8>, value: i32) {
        Self::write_unsigned_var_int(out, ((value << 1) ^ (value >> 31)) as u32);
    }

    // `BinaryStream::writeUnsignedVarInt`: 7 bits per byte, low bits first, the high bit set on every byte but the
    // last.
    fn write_unsigned_var_int(out: &mut Vec<u8>, mut value: u32) {
        while value >= 0x80 {
            out.push((value as u8 & 0x7F) | 0x80);
            value >>= 7;
        }
        out.push(value as u8);
    }

    // The variant index, then the type; for these actions the two are the same.
    fn write_kind(out: &mut Vec<u8>, kind: u8) {
        out.push(kind);
        out.push(kind);
    }

    // The container, no dynamic id, the slot, the net id as a little-endian signed int.
    fn write_slot(out: &mut Vec<u8>, slot: SlotRef) {
        out.push(slot.container);
        out.push(0);
        out.push(slot.slot);
        out.extend_from_slice(&slot.net_id.to_le_bytes());
    }
}
