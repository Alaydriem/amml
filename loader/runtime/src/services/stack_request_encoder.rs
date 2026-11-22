use crate::{ProtocolValues, RequestAction, SlotRef};

/// Writes the payload of item stack requests the way the game's `BinaryStream` writers do, so that `Packet::read` of
/// an `ItemStackRequestPacket` reads it: the number of requests, then for each its id, its actions, the strings to
/// filter (none) and their origin (-1). Each action starts with its index in the game's action variant, then its
/// type. The `93 01` packet header is not written: the loader fills a packet the game made from the payload.
pub struct StackRequestEncoder;

impl StackRequestEncoder {
    const ACTION_PLACE: u8 = 1;
    const ACTION_SWAP: u8 = 2;
    const ACTION_DROP: u8 = 3;
    const ACTION_TAKE: u8 = 0;
    const ACTION_DESTROY: u8 = 4;
    // `ItemStackRequestActionType::CraftCreative`, and its index in the game's action variant: the deprecated types 7 and 8 have
    // no struct in the variant, so the index is 12 (type 14) on 26.63 and on 70.26.
    const ACTION_CRAFT_CREATIVE: u8 = 14;
    const ACTION_CRAFT_CREATIVE_INDEX: u8 = 12;
    const CRAFTS: u8 = 1;
    const NOT_RANDOMLY: u8 = 0;
    const ORIGIN_NONE: i32 = -1;

    /// Several requests in one payload, in order, each with its id and its one action. The server answers each by
    /// its id.
    pub fn batch(requests: &[(i32, RequestAction)]) -> Vec<u8> {
        let mut out = Vec::with_capacity(4 + requests.len() * 28);
        Self::write_unsigned_var_int(&mut out, u32::try_from(requests.len()).unwrap_or(u32::MAX));
        for (request_id, action) in requests {
            Self::write_var_int(&mut out, *request_id);
            Self::write_unsigned_var_int(&mut out, Self::action_count(*action));
            Self::write_action(&mut out, *request_id, *action);
            Self::write_unsigned_var_int(&mut out, 0);
            out.extend_from_slice(&Self::ORIGIN_NONE.to_le_bytes());
        }
        out
    }

    // The creative craft is two actions (the craft, then the take of what it made); every other request has one.
    fn action_count(action: RequestAction) -> u32 {
        match action {
            RequestAction::CraftCreative { replacing: Some(_), .. } => 3,
            RequestAction::CraftCreative { .. } => 2,
            _ => 1,
        }
    }

    fn write_action(out: &mut Vec<u8>, request_id: i32, action: RequestAction) {
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
            RequestAction::CraftCreative { creative_net_id, count, to, replacing } => {
                if let Some((held_count, held)) = replacing {
                    Self::write_kind(out, Self::ACTION_DESTROY);
                    out.push(held_count);
                    Self::write_slot(out, held);
                }
                out.push(Self::ACTION_CRAFT_CREATIVE_INDEX);
                out.push(Self::ACTION_CRAFT_CREATIVE);
                Self::write_unsigned_var_int(out, creative_net_id);
                out.push(Self::CRAFTS);
                Self::write_kind(out, Self::ACTION_TAKE);
                out.push(count);
                // The made stack has no net id yet: the request's own id stands for it.
                let created = SlotRef {
                    container: ProtocolValues::CREATED_OUTPUT,
                    slot: ProtocolValues::CREATED_OUTPUT_SLOT,
                    net_id: request_id,
                };
                Self::write_slot(out, created);
                Self::write_slot(out, SlotRef { net_id: 0, ..to });
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
