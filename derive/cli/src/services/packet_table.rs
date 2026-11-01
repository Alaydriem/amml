use std::collections::BTreeMap;

use iced_x86::{Decoder, DecoderOptions, FlowControl, Instruction, Mnemonic, OpKind, Register};

use crate::errors::StepError;
use crate::structs::{Image, PacketList, PacketRow};

// how far each reader decodes before it gives up
const SWITCH_BYTES: usize = 128;
const CASE_STUB_BYTES: usize = 16;
const CASE_BYTES: usize = 4096;
const LEAF_BYTES: usize = 64;
// the std::make_shared control block in front of the packet
const CONTROL_BLOCK: u32 = 0x10;
const MAX_NAME: usize = 256;

/// Reads every packet that `MinecraftPackets::createPacket` makes, from its jump table. Each case calls a case
/// function that allocates the packet with `make_shared` (`mov edx, size+10h` before the allocator call), loads the
/// packet vtable (slot 1 is `getId`: `mov eax, id; ret`; slot 2 is `getName`) and loads the static handler object,
/// whose vtable slot 1 is a stub that calls `NetEventCallback` slot `disp/8` (`mov rax, [rax+disp]`).
pub struct PacketTable;

impl PacketTable {
    pub fn read(image: &Image, create_packet: u32) -> Result<PacketList, StepError> {
        let (targets, default) = Self::jump_table(image, create_packet)?;
        let mut list = PacketList { rows: Vec::new(), cases: targets.len() as u32, empty: 0, skipped: 0 };
        for target in targets {
            if Some(target) == default {
                list.empty += 1;
                continue;
            }
            match Self::case(image, target) {
                Some(row) => list.rows.push(row),
                None => list.skipped += 1,
            }
        }
        list.rows.sort_by_key(|row| row.id);
        Ok(list)
    }

    /// One line per packet, `id  name  vtable  getId  size  slot` (`?` when a value was not found), then the counts.
    /// With the old build, one line per name: `name  old id  new id  old size  new size  old slot  new slot  changed`.
    pub fn render(new: &PacketList, old: Option<&PacketList>) -> String {
        let value = |value: Option<u32>| value.map_or_else(|| "?".to_string(), |value| format!("{value:#x}"));
        let counts = |list: &PacketList| {
            format!(
                "cases: {}, rows: {}, empty: {}, skipped: {}",
                list.cases,
                list.rows.len(),
                list.empty,
                list.skipped
            )
        };
        let Some(old) = old else {
            let mut lines: Vec<String> = new
                .rows
                .iter()
                .map(|row| {
                    format!(
                        "{}  {}  {:#x}  {:#x}  {}  {}",
                        row.id,
                        row.name.as_deref().unwrap_or("-"),
                        row.vtable,
                        row.get_id,
                        value(row.size),
                        row.slot.map_or_else(|| "?".to_string(), |slot| slot.to_string()),
                    )
                })
                .collect();
            lines.push(counts(new));
            return lines.join("\n");
        };
        let key = |row: &PacketRow| row.name.clone().unwrap_or_else(|| format!("#{}", row.id));
        let mut joined: BTreeMap<String, (Option<&PacketRow>, Option<&PacketRow>)> = BTreeMap::new();
        for row in &old.rows {
            joined.entry(key(row)).or_default().0 = Some(row);
        }
        for row in &new.rows {
            joined.entry(key(row)).or_default().1 = Some(row);
        }
        let id = |row: Option<&PacketRow>| row.map_or_else(|| "-".to_string(), |row| row.id.to_string());
        let size = |row: Option<&PacketRow>| row.map_or_else(|| "-".to_string(), |row| value(row.size));
        let slot = |row: Option<&PacketRow>| {
            row.map_or_else(|| "-".to_string(), |row| row.slot.map_or_else(|| "?".to_string(), |slot| slot.to_string()))
        };
        let mut changed = 0;
        let mut lines = Vec::new();
        for (name, (before, after)) in &joined {
            let same = match (before, after) {
                (Some(a), Some(b)) => a.id == b.id && a.size == b.size && a.slot == b.slot,
                _ => false,
            };
            if !same {
                changed += 1;
            }
            lines.push(format!(
                "{name}  {}  {}  {}  {}  {}  {}  {}",
                id(*before),
                id(*after),
                size(*before),
                size(*after),
                slot(*before),
                slot(*after),
                if same { "no" } else { "yes" }
            ));
        }
        lines.push(format!("old {}; new {}; changed: {changed}", counts(old), counts(new)));
        lines.join("\n")
    }

    // the case targets of the switch and its default target (the `ja` past the bound)
    fn jump_table(image: &Image, create_packet: u32) -> Result<(Vec<u32>, Option<u32>), StepError> {
        let mut bound = None;
        let mut table = None;
        let mut default = None;
        for instruction in Self::decode(image, create_packet, SWITCH_BYTES) {
            match instruction.mnemonic() {
                Mnemonic::Cmp if instruction.op1_kind() == OpKind::Immediate32 => {
                    bound = u32::try_from(instruction.immediate(1)).ok();
                }
                Mnemonic::Ja if default.is_none() => default = u32::try_from(instruction.near_branch_target()).ok(),
                Mnemonic::Lea if instruction.is_ip_rel_memory_operand() => {
                    table = u32::try_from(instruction.ip_rel_memory_address()).ok();
                }
                Mnemonic::Jmp if instruction.op0_kind() == OpKind::Register => break,
                _ => {}
            }
        }
        let (Some(bound), Some(table)) = (bound, table) else {
            return Err(StepError::NoOperand { rva: create_packet, index: 0 });
        };
        let mut targets = Vec::new();
        for case in 0..=bound {
            let entry = image.read_u32(table + case * 4).ok_or(StepError::BadInstruction(table))? as i32;
            targets.push(table.wrapping_add_signed(entry));
        }
        Ok((targets, default))
    }

    // the packet of one case: the case stub calls the case function, or holds its code inline
    fn case(image: &Image, stub: u32) -> Option<PacketRow> {
        let call = Self::decode(image, stub, CASE_STUB_BYTES)
            .into_iter()
            .take_while(|instruction| instruction.flow_control() != FlowControl::Return)
            .find(|instruction| {
                instruction.flow_control() == FlowControl::Call && instruction.op0_kind() == OpKind::NearBranch64
            });
        let function = call.and_then(|call| u32::try_from(call.near_branch_target()).ok()).unwrap_or(stub);
        let mut row: Option<PacketRow> = None;
        let mut slot = None;
        let mut alloc = None;
        let mut size_mov: Option<(u32, usize)> = None;
        let mut callees = Vec::new();
        for (index, instruction) in Self::decode(image, function, CASE_BYTES).into_iter().enumerate() {
            if instruction.flow_control() == FlowControl::Return {
                break;
            }
            if instruction.mnemonic() == Mnemonic::Mov
                && instruction.op0_register() == Register::EDX
                && instruction.op1_kind() == OpKind::Immediate32
            {
                size_mov = Some((instruction.immediate32(), index));
            }
            if matches!(instruction.flow_control(), FlowControl::Call | FlowControl::IndirectCall)
                && alloc.is_none()
                && let Some((value, at)) = size_mov
                && index - at <= 2
            {
                alloc = Some(value);
            }
            if instruction.flow_control() == FlowControl::Call && instruction.op0_kind() == OpKind::NearBranch64 {
                callees.extend(u32::try_from(instruction.near_branch_target()).ok());
            }
            if instruction.mnemonic() != Mnemonic::Lea || !instruction.is_ip_rel_memory_operand() {
                continue;
            }
            let Ok(target) = u32::try_from(instruction.ip_rel_memory_address()) else { continue };
            if row.is_none() {
                row = Self::packet_vtable(image, target);
            }
            if slot.is_none() {
                slot = Self::handler_slot(image, target);
            }
        }
        // a case function that calls the packet constructor loads the vtable one call deeper
        let row = row.or_else(|| callees.into_iter().find_map(|callee| Self::vtable_in(image, callee)));
        row.map(|row| PacketRow { size: alloc.and_then(|alloc| alloc.checked_sub(CONTROL_BLOCK)), slot, ..row })
    }

    // the first packet vtable that a `lea` in `function` loads, up to its first `ret`
    fn vtable_in(image: &Image, function: u32) -> Option<PacketRow> {
        Self::decode(image, function, CASE_BYTES)
            .into_iter()
            .take_while(|instruction| instruction.flow_control() != FlowControl::Return)
            .filter(|instruction| instruction.mnemonic() == Mnemonic::Lea && instruction.is_ip_rel_memory_operand())
            .find_map(|instruction| {
                u32::try_from(instruction.ip_rel_memory_address())
                    .ok()
                    .and_then(|target| Self::packet_vtable(image, target))
            })
    }

    // a vtable whose slot 1 is `mov eax, imm32; ret`
    fn packet_vtable(image: &Image, vtable: u32) -> Option<PacketRow> {
        if image.section_of(vtable)?.executable {
            return None;
        }
        let get_id = image.code_pointer(vtable.checked_add(8)?)?;
        let bytes = image.read(get_id, 6)?;
        if bytes[0] != 0xB8 || bytes[5] != 0xC3 {
            return None;
        }
        let id = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
        let name = image.code_pointer(vtable.checked_add(16)?).and_then(|get_name| Self::name(image, get_name));
        Some(PacketRow { id, vtable, get_id, name, size: None, slot: None })
    }

    // `getName` writes a string_view: `lea reg, [rip+text]` and `mov qword ptr [rdx+8], length`
    fn name(image: &Image, get_name: u32) -> Option<String> {
        let mut text = None;
        let mut length = None;
        for instruction in Self::decode(image, get_name, LEAF_BYTES) {
            if instruction.flow_control() == FlowControl::Return {
                break;
            }
            if instruction.mnemonic() == Mnemonic::Lea && instruction.is_ip_rel_memory_operand() {
                text = u32::try_from(instruction.ip_rel_memory_address()).ok();
            }
            if instruction.mnemonic() == Mnemonic::Mov
                && instruction.op0_kind() == OpKind::Memory
                && instruction.memory_displacement64() == 8
                && matches!(instruction.op1_kind(), OpKind::Immediate32to64 | OpKind::Immediate32)
            {
                length = usize::try_from(instruction.immediate(1)).ok();
            }
        }
        let text = text?;
        let section = image.section_of(text).filter(|section| !section.executable)?;
        let tail = section.data.get(usize::try_from(text - section.rva).ok()?..)?;
        let end = match length {
            Some(length) if length <= MAX_NAME => length,
            Some(_) => return None,
            None => tail.iter().take(MAX_NAME).position(|&byte| byte == 0)?,
        };
        let bytes = tail.get(..end)?;
        (!bytes.is_empty() && bytes.iter().all(|byte| (0x20..=0x7E).contains(byte)))
            .then(|| String::from_utf8_lossy(bytes).into_owned())
    }

    // a static object whose vtable slot 1 is a stub with `mov reg, [reg+disp]`: the handler calls slot disp/8
    fn handler_slot(image: &Image, object: u32) -> Option<u32> {
        let vtable = u32::try_from(image.read_u64(object)?.checked_sub(image.image_base())?).ok()?;
        if image.section_of(vtable)?.executable {
            return None;
        }
        let stub = image.code_pointer(vtable.checked_add(8)?)?;
        Self::decode(image, stub, LEAF_BYTES)
            .into_iter()
            .take_while(|instruction| !matches!(instruction.flow_control(), FlowControl::Return))
            .find(|instruction| {
                instruction.mnemonic() == Mnemonic::Mov
                    && instruction.op0_kind() == OpKind::Register
                    && instruction.op1_kind() == OpKind::Memory
                    && instruction.memory_base() == instruction.op0_register()
                    && instruction.memory_index() == Register::None
                    && instruction.memory_displacement64() > 0
                    && instruction.memory_displacement64() % 8 == 0
            })
            .and_then(|instruction| u32::try_from(instruction.memory_displacement64() / 8).ok())
    }

    // decodes from `rva` to an invalid instruction, the end of its section or `max` bytes
    fn decode(image: &Image, rva: u32, max: usize) -> Vec<Instruction> {
        let Some(section) = image.section_of(rva).filter(|section| section.executable) else { return Vec::new() };
        let start = (rva - section.rva) as usize;
        let end = section.data.len().min(start.saturating_add(max));
        let Some(bytes) = section.data.get(start..end) else { return Vec::new() };
        Decoder::with_ip(64, bytes, u64::from(rva), DecoderOptions::NONE)
            .into_iter()
            .take_while(|instruction| !instruction.is_invalid())
            .collect()
    }
}
