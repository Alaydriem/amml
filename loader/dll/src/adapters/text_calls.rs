use std::ffi::c_void;

use amml_abi::AmmlStatus;

use crate::{DrawCalls, FontFrame, LoaderState, ObjectRead, SessionAdapter};

/// `FontHandle ClientInstance::getFontHandle() const`: rcx the client, rdx the hidden result buffer.
type FontHandleFn = unsafe extern "C" fn(usize, *mut c_void) -> *mut c_void;

/// The entries the text calls (`UiDraw::text_width`, `UiDraw::text`) use.
///
/// `draw_slot` and `flush_slot` are slots of the `MinecraftUIRenderContext` vtable, `get_font_handle_slot` a slot of
/// the `ClientInstance` vtable and `line_length_slot` a slot of the `Font` vtable. `font_dtor` is the address of the
/// non-deleting `FontHandle` destructor. `font_handle_size` and `default_font` are the size of a `FontHandle` and the
/// offset of its default font, `loaded_fonts` the offset of the font vector in a `FontRepository`.
#[derive(Clone, Copy, Debug)]
pub struct TextCalls {
    pub draw_slot: usize,
    pub flush_slot: usize,
    pub get_font_handle_slot: usize,
    pub line_length_slot: usize,
    pub font_dtor: usize,
    pub font_handle_size: usize,
    pub default_font: usize,
    pub loaded_fonts: usize,
}

impl TextCalls {
    /// The size of the `FontHandle` mirror: the loader reads the handle at fixed offsets only when the build agrees.
    pub const HANDLE_SIZE: usize = 0x50;
    /// The offset of the default font (`shared_ptr<Font>`) of a `FontHandle` on a build the mirror fits.
    pub const DEFAULT_FONT_OFFSET: usize = 0x30;
    /// The offset of the repository pointer in a `FontHandle`: the `NonOwnerPointer` holds its control block at 0x18
    /// and the pointer at 0x28.
    pub const REPOSITORY_OFFSET: usize = 0x28;
    /// The offset of the font id (an index into the repository) in a `FontHandle`.
    pub const FONT_ID_OFFSET: usize = 0x40;
    /// The vector of a `FontRepository` is after its bases and `mIsInitialized`: no offset below this, or above
    /// `MAX_LOADED_FONTS_OFFSET`, is the vector.
    pub const MIN_LOADED_FONTS_OFFSET: usize = 0x38;
    pub const MAX_LOADED_FONTS_OFFSET: usize = 0x100;
    // The most fonts a repository holds that the loader believes, and the size of one element of its vector.
    const MAX_LOADED_FONTS: usize = 4096;
    const FONT_ELEMENT: usize = 16;

    /// The calls from the resolved entries; `None` when one is missing or a layout entry is not the mirror's value.
    pub fn new(state: &LoaderState) -> Option<Self> {
        let value = |name: &str| state.resolve(name).ok().and_then(|value| usize::try_from(value).ok());
        let calls = Self {
            draw_slot: value("MinecraftUIRenderContext.drawText")?,
            flush_slot: value("MinecraftUIRenderContext.flushText")?,
            get_font_handle_slot: value("ClientInstance.getFontHandle")?,
            line_length_slot: value("Font.getLineLength")?,
            font_dtor: value("FontHandle.dtor").filter(|address| *address != 0)?,
            font_handle_size: value("FontHandle.size")?,
            default_font: value("FontHandle.mDefaultFont")?,
            loaded_fonts: value("FontRepository.mLoadedFonts")?,
        };
        calls.fits().then_some(calls)
    }

    /// Whether the layout values are the ones the `FontHandle` and `FontRepository` reads are written for.
    pub fn fits(&self) -> bool {
        self.font_handle_size == Self::HANDLE_SIZE
            && self.default_font == Self::DEFAULT_FONT_OFFSET
            && (Self::MIN_LOADED_FONTS_OFFSET..=Self::MAX_LOADED_FONTS_OFFSET).contains(&self.loaded_fonts)
    }

    /// Asks the primary client for its font handle and picks the font of it, in a frame that destroys the handle when it
    /// drops. `NOT_AVAILABLE` without a primary client, a slot or a font. Game thread only.
    pub(crate) fn open_frame(&self, serial: u64) -> Result<FontFrame, AmmlStatus> {
        let client = SessionAdapter::primary_client().ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the primary client is a live `ClientInstance`; its vtable holds `getFontHandle` at the slot of the
        // entry.
        let function =
            unsafe { DrawCalls::slot(client, self.get_font_handle_slot) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the slot holds `getFontHandle`, whose arguments are these.
        let function = unsafe { std::mem::transmute::<usize, FontHandleFn>(function) };
        let buffer = Box::new([0; Self::HANDLE_SIZE / size_of::<u64>()]);
        let mut frame = FontFrame::new(serial, buffer, self.font_dtor);
        // SAFETY: the buffer is `FontHandle.size` bytes and 8-byte aligned; the client is live. The frame holds the
        // buffer from here, so every later return destroys the handle the game made.
        unsafe { function(client, frame.buffer()) };
        frame.font = self.pick_font(frame.address()).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        Ok(frame)
    }

    // The `Font*` of the handle at `handle`, picked as `FontHandle::getFont` picks it: the font at index `mFontId`
    // of the repository when the id is in range, else the default font. `None` when no font is there.
    fn pick_font(&self, handle: usize) -> Option<usize> {
        // SAFETY: `handle` is the handle the game just filled; the offsets are the mirror's (`Self::fits`).
        let id = unsafe { ObjectRead::value::<u64>(handle, Self::FONT_ID_OFFSET) }?;
        // SAFETY: as above; the repository pointer is null or a live repository.
        let repository = unsafe { ObjectRead::pointer(handle, Self::REPOSITORY_OFFSET) };
        let picked = repository.and_then(|repository| {
            // SAFETY: a live repository holds the begin and end pointers of its font vector at the entry's offset.
            let begin = unsafe { ObjectRead::pointer(repository, self.loaded_fonts) }?;
            // SAFETY: as above, the end pointer follows the begin pointer.
            let end = unsafe { ObjectRead::pointer(repository, self.loaded_fonts + size_of::<usize>()) }?;
            let bytes = end.checked_sub(begin)?;
            let count = bytes / Self::FONT_ELEMENT;
            let index = usize::try_from(id).ok()?;
            if bytes % Self::FONT_ELEMENT != 0 || count > Self::MAX_LOADED_FONTS || index >= count {
                return None;
            }
            // SAFETY: the element is inside the vector; its first word is the `Font*` of the `shared_ptr`.
            unsafe { ObjectRead::pointer(begin, index * Self::FONT_ELEMENT) }
        });
        // SAFETY: the default font is a `shared_ptr<Font>` of the handle; its first word is the `Font*`.
        picked.or_else(|| unsafe { ObjectRead::pointer(handle, self.default_font) })
    }
}
