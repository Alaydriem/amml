use std::collections::BTreeMap;

use amml_data::{Check, EntryId, EntryKind, Resolved};
use iced_x86::{Decoder, DecoderOptions, FlowControl, Instruction};

use crate::services::{CodeXref, LayoutGuard, RttiIndex};
use crate::structs::{CodeRefKind, Image};

pub struct CheckRunner;

impl CheckRunner {
    // the function alignment of the game's linker
    const LEAF_ALIGNMENT: u32 = 16;
    const INT3: u8 = 0xCC;

    /// A check that does not fit the kind of the value (for example `FunctionStart` on a field offset) can only return false.
    /// `resolved` holds the entries found so far; a check that reads one (`in_vtable`) fails when it is missing.
    pub fn run(
        image: &Image,
        rtti: &RttiIndex,
        check: &Check,
        _kind: EntryKind,
        value: u64,
        resolved: &BTreeMap<EntryId, Resolved>,
    ) -> bool {
        match check {
            Check::FunctionStart => u32::try_from(value).is_ok_and(|rva| image.functions().is_start(rva)),
            Check::LeafStart => u32::try_from(value).is_ok_and(|rva| Self::leaf_start(image, rva)),
            Check::InVtableOf { class } => u32::try_from(value).is_ok_and(|rva| rtti.in_vtable(image, class, rva)),
            Check::InVtable { vtable, min_slot, max_slot } => {
                let (Ok(rva), Some(table)) = (u32::try_from(value), resolved.get(vtable)) else { return false };
                u32::try_from(table.value).is_ok_and(|table| Self::in_slots(image, table, rva, *min_slot, *max_slot))
            }
            Check::Section { name } => {
                u32::try_from(value).is_ok_and(|rva| image.section_of(rva).is_some_and(|section| &section.name == name))
            }
            Check::Range { min, max } => *min <= value && value <= *max,
            Check::EntryIs { entry, value: wanted } => {
                LayoutGuard::holds(*wanted, resolved.get(entry).map(|other| other.value))
            }
        }
    }

    // True when `rva` starts code that is called directly: in an executable section, 16-byte aligned, after int3
    // padding or a ret / jmp that ends a `.pdata` range, inside no `.pdata` range of another function, and the target
    // of at least one confirmed `call rel32`
    fn leaf_start(image: &Image, rva: u32) -> bool {
        let functions = image.functions();
        let executable = image.section_of(rva).is_some_and(|section| section.executable);
        let own_range = functions.containing(rva).is_none() || functions.is_start(rva);
        executable
            && rva.is_multiple_of(Self::LEAF_ALIGNMENT)
            && own_range
            && Self::after_padding_or_exit(image, rva)
            && CodeXref::new(image).callers(rva).iter().any(|caller| caller.kind == CodeRefKind::Call)
    }

    // True when the byte before `rva` is int3 padding, or a `.pdata` range ends at `rva` with a ret or a jmp
    fn after_padding_or_exit(image: &Image, rva: u32) -> bool {
        let Some(before) = rva.checked_sub(1) else { return false };
        if image.read(before, 1) == Some(&[Self::INT3]) {
            return true;
        }
        let functions = image.functions();
        let Some((primary, end)) = functions.containing(before) else { return false };
        let Some((start, _)) =
            functions.ranges_of(primary).into_iter().find(|&(from, to)| from <= before && before < to)
        else {
            return false;
        };
        if end != rva {
            return false;
        }
        let Some(bytes) = image.read(start, (end - start) as usize) else { return false };
        let mut decoder = Decoder::with_ip(64, bytes, u64::from(start), DecoderOptions::NONE);
        let mut instruction = Instruction::default();
        let mut last = None;
        while decoder.can_decode() {
            decoder.decode_out(&mut instruction);
            if instruction.is_invalid() {
                return false;
            }
            last = Some(instruction.flow_control());
        }
        matches!(last, Some(FlowControl::Return | FlowControl::UnconditionalBranch | FlowControl::IndirectBranch))
    }

    // True when `function` is the code target of a slot in `min..=max`; a slot up to `max` that is not code fails the check
    fn in_slots(image: &Image, table: u32, function: u32, min: u32, max: u32) -> bool {
        let mut found = false;
        // every slot from 0 up to `max` must be code, as `slot_of` reads them, even those below `min`
        for slot in 0..=max {
            let Some(target) =
                slot.checked_mul(8).and_then(|d| table.checked_add(d)).and_then(|rva| image.code_pointer(rva))
            else {
                return false;
            };
            found |= slot >= min && target == function;
        }
        found
    }
}
