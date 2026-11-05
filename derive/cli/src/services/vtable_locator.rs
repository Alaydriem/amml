use std::collections::BTreeSet;
use std::sync::OnceLock;

use iced_x86::{Decoder, DecoderOptions, Instruction, Mnemonic, OpKind, Register};
use memchr::memchr_iter;

use super::CodeXref;
use crate::errors::StepError;
use crate::structs::{Image, VtableAnchors};

/// Finds the start of the one vtable that holds a pointer to a function.
///
/// A table start is an address that code stores as a vptr: `lea reg,[rip+disp]` naming it, then
/// `mov [base],reg` (displacement 0, the same 64-bit register) within the next `STORE_WINDOW`
/// instructions. The set of those addresses is built on first use, in one pass over the executable
/// sections, by decoding at each byte that looks like such a `lea`; no `.pdata` range is needed.
///
/// The same pass also collects every target of a RIP-relative `lea` as a barrier. The walk back from a slot
/// stops at the first barrier, which must be a vptr-store start; a barrier that is not one (a `lea` of the
/// table whose store has another form) leaves the slot without a start. A slot without a start refuses the
/// step: that table may be the right one.
///
/// Known limits: a `lea` found at a misaligned byte (inside another instruction) counts like a real one, as a
/// barrier, and as a start when a store follows its bytes. A table with no RIP-relative `lea` that the pass
/// decodes has no barrier, so a walk from it can reach the table before.
pub struct VtableLocator<'a> {
    image: &'a Image,
    anchors: OnceLock<VtableAnchors>,
}

impl<'a> VtableLocator<'a> {
    // instructions after the `lea` that may hold the store
    const STORE_WINDOW: usize = 3;
    // bytes decoded from a `lea`: the `lea` and three more instructions
    const DECODE_BYTES: usize = 64;
    // qwords the walk back from a slot reads before it gives up
    const WALK_LIMIT: u32 = 1024;

    pub fn new(image: &'a Image) -> Self {
        Self { image, anchors: OnceLock::new() }
    }

    /// The start of the single vtable with a slot that points to `function`.
    pub fn table_holding(&self, function: u32) -> Result<u32, StepError> {
        let anchors = self.anchors.get_or_init(|| self.build_anchors());
        let mut tables = BTreeSet::new();
        for pointer in CodeXref::new(self.image).data_pointers(function) {
            match self.table_start(anchors, pointer.site) {
                Some(start) => {
                    tables.insert(start);
                }
                None => return Err(StepError::UnreadableTable { rva: function, slot: pointer.site }),
            }
        }
        let mut found = tables.iter();
        match (found.next(), found.next()) {
            (None, _) => Err(StepError::NoVtableHolds { rva: function }),
            (Some(only), None) => Ok(*only),
            _ => Err(StepError::InSeveralVtables { rva: function, count: tables.len() }),
        }
    }

    // walks back from `slot` over code pointers to the first barrier, which is the table start when it is a vptr
    // store target; None when that barrier is not, or a qword that is not a code pointer or the limit comes first
    fn table_start(&self, anchors: &VtableAnchors, slot: u32) -> Option<u32> {
        for index in 0..Self::WALK_LIMIT {
            let at = slot.checked_sub(index * 8)?;
            self.image.code_pointer(at)?;
            if anchors.barriers.contains(&at) {
                return anchors.starts.contains(&at).then_some(at);
            }
        }
        None
    }

    fn build_anchors(&self) -> VtableAnchors {
        let mut anchors = VtableAnchors { starts: BTreeSet::new(), barriers: BTreeSet::new() };
        for section in self.image.sections().iter().filter(|s| s.executable) {
            // a 64-bit `lea reg,[rip+disp32]` is REX.W (0x48..=0x4F), 0x8D, ModRM with mod 00 and rm 101
            for at in memchr_iter(0x8D, &section.data) {
                let Some(rex_at) = at.checked_sub(1) else { continue };
                let is_lea = section.data.get(rex_at).is_some_and(|rex| rex & 0xF8 == 0x48)
                    && section.data.get(at + 1).is_some_and(|modrm| modrm & 0xC7 == 0x05);
                if !is_lea {
                    continue;
                }
                let Some(rva) = u32::try_from(rex_at).ok().and_then(|offset| section.rva.checked_add(offset)) else {
                    continue;
                };
                let end = (rex_at + Self::DECODE_BYTES).min(section.data.len());
                if let Some((target, stored)) = Self::lea_target(rva, &section.data[rex_at..end]) {
                    anchors.barriers.insert(target);
                    if stored {
                        anchors.starts.insert(target);
                    }
                }
            }
        }
        anchors
    }

    // the address a `lea reg,[rip+disp]` at the start of `bytes` names, and whether a store of `reg` through
    // `[base]` follows within the window
    fn lea_target(rva: u32, bytes: &[u8]) -> Option<(u32, bool)> {
        let mut decoder = Decoder::with_ip(64, bytes, u64::from(rva), DecoderOptions::NONE);
        let lea = decoder.decode();
        let register = lea.op0_register();
        let clean = lea.mnemonic() == Mnemonic::Lea
            && lea.len() == 7
            && lea.op0_kind() == OpKind::Register
            && register.is_gpr64()
            && lea.is_ip_rel_memory_operand();
        if !clean {
            return None;
        }
        let target = u32::try_from(lea.ip_rel_memory_address()).ok()?;
        let mut instruction = Instruction::default();
        for _ in 0..Self::STORE_WINDOW {
            if !decoder.can_decode() {
                break;
            }
            decoder.decode_out(&mut instruction);
            if instruction.is_invalid() {
                break;
            }
            if Self::is_vptr_store(&instruction, register) {
                return Some((target, true));
            }
        }
        Some((target, false))
    }

    fn is_vptr_store(instruction: &Instruction, register: Register) -> bool {
        instruction.mnemonic() == Mnemonic::Mov
            && instruction.op0_kind() == OpKind::Memory
            && instruction.op1_kind() == OpKind::Register
            && instruction.op1_register() == register
            && instruction.memory_displacement64() == 0
            && instruction.memory_index() == Register::None
            && instruction.memory_base() != Register::None
            && instruction.memory_base() != Register::RIP
    }
}
