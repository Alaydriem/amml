use std::ffi::c_void;
use std::fmt::Write;

use amml_game::{CompoundTagLayout, CompoundTagRef, CompoundTagVariantLayout, ListTagLayout, RawObject};
use amml_msvc::MsvcString;

/// Writes a game NBT tree as text, the way `/data` prints it: `{key:value, ...}`, lists in `[...]`, a byte with `b`,
/// a short with `s`, a long with `L`, a float with `f`, a double with `d`. Only reads; no game call except
/// `Tag::getId` on list elements. The text stops at `MAX_CHARS` and the tree at `MAX_DEPTH`.
pub struct NbtDump {
    text: String,
}

impl NbtDump {
    /// The deepest tag written; a deeper one is written as `...`.
    const MAX_DEPTH: usize = 12;
    /// The longest text; the rest is cut and `...(cut)` added.
    const MAX_CHARS: usize = 12_000;
    // `Tag::getId` is the sixth virtual of `Tag`.
    const GET_ID_SLOT: u64 = 5;
    // The value of a numeric or string tag follows the vtable.
    const VALUE_OFFSET: u64 = 8;

    const BYTE: u8 = 1;
    const SHORT: u8 = 2;
    const INT: u8 = 3;
    const LONG: u8 = 4;
    const FLOAT: u8 = 5;
    const DOUBLE: u8 = 6;
    const BYTE_ARRAY: u8 = 7;
    const STRING: u8 = 8;
    const LIST: u8 = 9;
    const COMPOUND: u8 = 10;
    const INT_ARRAY: u8 = 11;

    /// The text of the tree under `tag`.
    pub fn of(tag: &CompoundTagRef<'_>) -> String {
        let mut dump = Self { text: String::new() };
        dump.write_tag(tag.as_ptr(), Self::COMPOUND, 0);
        if dump.text.len() > Self::MAX_CHARS {
            let mut end = Self::MAX_CHARS;
            while !dump.text.is_char_boundary(end) {
                end -= 1;
            }
            dump.text.truncate(end);
            dump.text.push_str("...(cut)");
        }
        dump.text
    }

    fn full(&self) -> bool {
        self.text.len() > Self::MAX_CHARS
    }

    // Writes the tag object at `tag`, whose `Tag::Type` is `kind`.
    #[allow(unsafe_code)]
    fn write_tag(&mut self, tag: *const c_void, kind: u8, depth: usize) {
        if self.full() {
            return;
        }
        if tag.is_null() {
            self.text.push_str("null");
            return;
        }
        if depth > Self::MAX_DEPTH {
            self.text.push_str("...");
            return;
        }
        // SAFETY: `tag` is a live Tag of type `kind`; each arm reads the value its class holds after the vtable.
        unsafe {
            match kind {
                Self::BYTE => self.number(RawObject::read::<i8>(tag, Self::VALUE_OFFSET), "b"),
                Self::SHORT => self.number(RawObject::read::<i16>(tag, Self::VALUE_OFFSET), "s"),
                Self::INT => self.number(RawObject::read::<i32>(tag, Self::VALUE_OFFSET), ""),
                Self::LONG => self.number(RawObject::read::<i64>(tag, Self::VALUE_OFFSET), "L"),
                Self::FLOAT => self.number(RawObject::read::<f32>(tag, Self::VALUE_OFFSET), "f"),
                Self::DOUBLE => self.number(RawObject::read::<f64>(tag, Self::VALUE_OFFSET), "d"),
                Self::STRING => {
                    let text = &*tag.cast::<u8>().add(Self::VALUE_OFFSET as usize).cast::<MsvcString>();
                    let _ = write!(self.text, "{:?}", String::from_utf8_lossy(text.as_bytes()));
                }
                Self::LIST => self.list(tag, depth),
                Self::COMPOUND => self.compound(tag, depth),
                Self::BYTE_ARRAY => self.text.push_str("<byte array>"),
                Self::INT_ARRAY => self.text.push_str("<int array>"),
                other => {
                    let _ = write!(self.text, "<type {other}>");
                }
            }
        }
    }

    fn number<T: std::fmt::Display>(&mut self, value: Option<T>, suffix: &str) {
        match value {
            Some(value) => {
                let _ = write!(self.text, "{value}{suffix}");
            }
            None => self.text.push('?'),
        }
    }

    #[allow(unsafe_code)]
    fn compound(&mut self, tag: *const c_void, depth: usize) {
        // SAFETY: the tag is a live CompoundTag, which the mirror fits.
        let layout = unsafe { &*tag.cast::<CompoundTagLayout>() };
        let mut children: Vec<(String, *const c_void, u8)> = Vec::new();
        // SAFETY: the tag's map is a live std::map<std::string, CompoundTagVariant>.
        let complete = unsafe {
            layout.tags.for_each(|key, value: &CompoundTagVariantLayout| {
                children.push((String::from_utf8_lossy(key).into_owned(), value.tag(), value.index));
            })
        };
        self.text.push('{');
        for (position, (key, child, kind)) in children.iter().enumerate() {
            if position > 0 {
                self.text.push_str(", ");
            }
            self.text.push_str(key);
            self.text.push(':');
            self.write_tag(*child, *kind, depth + 1);
        }
        if !complete {
            self.text.push_str(" <walk failed>");
        }
        self.text.push('}');
    }

    #[allow(unsafe_code)]
    fn list(&mut self, tag: *const c_void, depth: usize) {
        // SAFETY: the tag is a live ListTag, which the mirror fits.
        let layout = unsafe { &*tag.cast::<ListTagLayout>() };
        let elements = layout.items.as_slice();
        self.text.push('[');
        for (position, element) in elements.iter().enumerate() {
            if position > 0 {
                self.text.push_str(", ");
            }
            let element = element.cast_const();
            // SAFETY: a non-null element of a live list is a live Tag, whose first qword is its vtable.
            let get_id = unsafe { RawObject::slot(element, Self::GET_ID_SLOT) };
            let Some(get_id) = get_id else {
                self.text.push_str("null");
                continue;
            };
            // SAFETY: the slot holds `Tag::Type Tag::getId() const`: this in rcx, the type in al.
            let get_id: unsafe extern "C" fn(*const c_void) -> u8 = unsafe { std::mem::transmute(get_id) };
            // SAFETY: the element is a live Tag.
            let kind = unsafe { get_id(element) };
            self.write_tag(element, kind, depth + 1);
        }
        self.text.push(']');
    }
}
