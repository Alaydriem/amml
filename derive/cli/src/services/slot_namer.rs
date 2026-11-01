use crate::services::StringXref;
use crate::structs::Image;

/// Names a function from the `virtual` `__FUNCSIG__` text that MSVC puts in its asserts and logs. Used to label vtable slots, whose order in the headers is not the order in the binary.
pub struct SlotNamer<'a> {
    image: &'a Image,
    xrefs: &'a StringXref,
}

impl<'a> SlotNamer<'a> {
    pub fn new(image: &'a Image, xrefs: &'a StringXref) -> Self {
        Self { image, xrefs }
    }

    /// The first loaded string that starts with `virtual ` and holds `::` and `(`: the `__FUNCSIG__` of a
    /// virtual, which is what a slot holds. Other strings are not names: inlined helpers load their own
    /// `__FUNCSIG__` text, which names the helper and not the slot. A function whose first instruction is
    /// `jmp rel32` is named from the jump target; only one jump is followed.
    pub fn name_of(&self, function: u32) -> Option<String> {
        let body = self.jump_target(function).unwrap_or(function);
        self.xrefs
            .strings_used_by(self.image, body)
            .into_iter()
            .find(|text| text.starts_with("virtual ") && text.contains("::") && text.contains('('))
    }

    // the target of a `jmp rel32` at `function`
    fn jump_target(&self, function: u32) -> Option<u32> {
        let bytes = self.image.read(function, 5)?;
        if bytes[0] != 0xE9 {
            return None;
        }
        let disp = i32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
        function.checked_add(5)?.checked_add_signed(disp)
    }
}
