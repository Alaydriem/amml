use crate::structs::Image;

pub struct VtableReader<'a> {
    image: &'a Image,
}

impl<'a> VtableReader<'a> {
    pub fn new(image: &'a Image) -> Self {
        Self { image }
    }

    /// Rows `(slot, target RVA, target starts a function)` for the slots in `from..to` of the vtable at
    /// `vtable_rva`. The read stops at the first slot that does not hold a code pointer.
    pub fn slots(&self, vtable_rva: u32, from: u32, to: u32) -> Vec<(u32, Option<u32>, bool)> {
        let mut rows = Vec::new();
        for slot in from..to {
            let at = slot.checked_mul(8).and_then(|offset| vtable_rva.checked_add(offset));
            let Some(target) = at.and_then(|at| self.image.code_pointer(at)) else {
                break;
            };
            rows.push((slot, Some(target), self.image.functions().is_start(target)));
        }
        rows
    }
}
