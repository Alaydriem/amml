use std::marker::PhantomData;

use amml_abi::{AmmlImageView, AmmlNetworkTable, AmmlStatus, AmmlStr, NetworkSkinFlagFn, NetworkSkinTextFn};

use crate::{AnimatedKind, ImageView, PixelFormat};

/// A skin the game holds, lent for one network event (`PlayerListUpdate` or `PlayerSkinChanged`). Every read gives a
/// view into the game's memory, or `None` when the build cannot read it (the area of the group is not offered), the
/// value is not valid, or the event is over. Nothing is copied.
#[derive(Clone, Copy)]
pub struct SkinRef<'f> {
    table: AmmlNetworkTable,
    handle: u64,
    _event: PhantomData<&'f ()>,
}

impl<'f> SkinRef<'f> {
    pub(crate) fn new(table: AmmlNetworkTable, handle: u64) -> Self {
        Self { table, handle, _event: PhantomData }
    }

    /// The main image (area `network.skin`); `None` when its blob is not `width * height * 4` bytes.
    #[allow(unsafe_code)]
    pub fn image(&self) -> Option<ImageView<'f>> {
        let mut out = AmmlImageView::EMPTY;
        // SAFETY: the table function is the loader's; `out` is writable.
        let status = unsafe { (self.table.skin_image)(self.handle, &raw mut out) };
        Self::view(status, out)
    }

    /// The animated images (area `network.skin_animated`) with their kinds, in the game's order. An image whose blob is
    /// not `width * height * 4` bytes is left out.
    #[allow(unsafe_code)]
    pub fn animated_images(&self) -> impl Iterator<Item = (AnimatedKind, ImageView<'f>)> + use<'f> {
        let skin = *self;
        let mut count = 0u32;
        // SAFETY: the table function is the loader's; `count` is writable.
        let status = unsafe { (skin.table.skin_animated_count)(skin.handle, &raw mut count) };
        let count = if status == AmmlStatus::OK { count } else { 0 };
        (0..count).filter_map(move |index| {
            let (mut kind, mut out) = (0u32, AmmlImageView::EMPTY);
            // SAFETY: the table function is the loader's; both outputs are writable.
            let status = unsafe { (skin.table.skin_animated)(skin.handle, index, &raw mut kind, &raw mut out) };
            Self::view(status, out).map(|view| (AnimatedKind::from(kind), view))
        })
    }

    /// Whether the skin is a persona skin (area `network.skin_details`).
    pub fn is_persona(&self) -> Option<bool> {
        self.flag(self.table.skin_is_persona)
    }

    /// Whether the skin is a premium skin (area `network.skin_details`).
    pub fn is_premium(&self) -> Option<bool> {
        self.flag(self.table.skin_is_premium)
    }

    /// The name of the default geometry (area `network.skin_geometry`); `None` when it is not UTF-8.
    pub fn default_geometry_name(&self) -> Option<&'f str> {
        self.text(self.table.skin_geometry_name)
    }

    /// The geometry as JSON text (area `network.skin_geometry`). The loader builds the text on the first read of each
    /// skin in an event, and keeps it until the event ends; it gives `None` above 1 MB or for text that is not UTF-8.
    pub fn geometry_json(&self) -> Option<&'f str> {
        self.text(self.table.skin_geometry_json)
    }

    /// The resource patch text (area `network.skin_details`).
    pub fn resource_patch(&self) -> Option<&'f str> {
        self.text(self.table.skin_resource_patch)
    }

    #[allow(unsafe_code)]
    fn flag(&self, read: NetworkSkinFlagFn) -> Option<bool> {
        let mut out = 0u8;
        // SAFETY: the table function is the loader's; `out` is writable.
        let status = unsafe { read(self.handle, &raw mut out) };
        (status == AmmlStatus::OK).then_some(out != 0)
    }

    #[allow(unsafe_code)]
    fn text(&self, read: NetworkSkinTextFn) -> Option<&'f str> {
        let mut out = AmmlStr::EMPTY;
        // SAFETY: the table function is the loader's; `out` is writable.
        let status = unsafe { read(self.handle, &raw mut out) };
        if status != AmmlStatus::OK {
            return None;
        }
        // SAFETY: the loader keeps the text readable until the event ends, and `'f` ends with the borrow of the event.
        unsafe { out.as_str() }.ok()
    }

    #[allow(unsafe_code)]
    fn view(status: AmmlStatus, out: AmmlImageView) -> Option<ImageView<'f>> {
        if status != AmmlStatus::OK || out.pixels.is_null() {
            return None;
        }
        // SAFETY: the loader lends `len` readable bytes until the event ends, and `'f` ends with the borrow of the event.
        let pixels = unsafe { std::slice::from_raw_parts(out.pixels, out.len) };
        Some(ImageView { width: out.width, height: out.height, format: PixelFormat::from(out.format), pixels })
    }
}
