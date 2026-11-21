use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicBool, Ordering};

use amml_abi::{AmmlImageView, AmmlStatus, AmmlStr};
use amml_msvc::{MsvcHeap, MsvcString};

use crate::{MceImageLayout, ObjectRead, SkinAnimatedLayout, SkinDetailsLayout, SkinGeometryLayout};

/// `std::string Json::Value::toStyledString() const`: rcx the value, rdx the hidden pointer to an uninitialised string
/// that the call builds and returns.
type ToStyledStringFn = unsafe extern "C" fn(usize, *mut MsvcString) -> *mut MsvcString;

static IMAGE_LOGGED: AtomicBool = AtomicBool::new(false);
static ANIMATED_LOGGED: AtomicBool = AtomicBool::new(false);

/// The reads of a `SerializedSkinImpl` behind a `SerializedSkinRef`, over the layouts the entries gave. Every function
/// takes the skin object (`element`, the first word of the `shared_ptr`, see `SkinReader::element`). The groups a build
/// does not have are `None`, and their reads give `NOT_AVAILABLE`.
///
/// Every read is valid only while the game keeps the skin alive: inside the handler that holds the packet.
pub struct SkinReader {
    image: Option<usize>,
    animated: Option<SkinAnimatedLayout>,
    geometry: Option<SkinGeometryLayout>,
    details: Option<SkinDetailsLayout>,
}

impl SkinReader {
    /// Longest text of the geometry a read gives; a longer one is freed unread.
    pub const MAX_JSON: usize = 1024 * 1024;
    /// Longest name or patch text a read gives; a longer one is a misread.
    pub const MAX_TEXT: usize = 64 * 1024;
    // Capacity of the inline buffer of a `std::string`.
    const INLINE: usize = 15;
    const STRING_SIZE_AT: usize = 0x10;
    const STRING_CAPACITY_AT: usize = 0x18;

    pub fn new(
        image: Option<usize>,
        animated: Option<SkinAnimatedLayout>,
        geometry: Option<SkinGeometryLayout>,
        details: Option<SkinDetailsLayout>,
    ) -> Self {
        Self { image, animated, geometry, details }
    }

    /// Whether the animated images can be read.
    pub fn has_animated(&self) -> bool {
        self.animated.is_some()
    }

    /// Whether the geometry can be read.
    pub fn has_geometry(&self) -> bool {
        self.geometry.is_some()
    }

    /// Whether the persona, premium and patch reads work.
    pub fn has_details(&self) -> bool {
        self.details.is_some()
    }

    /// The skin object behind the `shared_ptr` at `skin_ref`; `None` for a null pointer.
    ///
    /// # Safety
    /// `skin_ref` is the address of a live `SerializedSkinRef`.
    pub unsafe fn element(skin_ref: usize) -> Option<usize> {
        // SAFETY: the caller guarantees a live `shared_ptr`, whose first word is the element pointer.
        unsafe { ObjectRead::pointer(skin_ref, 0) }
    }

    /// The main image; `NOT_AVAILABLE` for a blob whose size is not `width * height * 4`.
    ///
    /// # Safety
    /// `element` is a live `SerializedSkinImpl`.
    pub unsafe fn image(&self, element: usize) -> Result<AmmlImageView, AmmlStatus> {
        let at = self.image.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the main image is a member of the live skin.
        let image = unsafe { ObjectRead::value::<MceImageLayout>(element, at) };
        Self::view(image, &IMAGE_LOGGED, "main")
    }

    /// The number of animated images, at most `SkinAnimatedLayout::MAX_IMAGES`.
    ///
    /// # Safety
    /// `element` is a live `SerializedSkinImpl`.
    pub unsafe fn animated_count(&self, element: usize) -> Result<u32, AmmlStatus> {
        let layout = self.animated.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the caller guarantees a live skin.
        let count = unsafe { Self::animated_len(&layout, element) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        Ok(count as u32)
    }

    /// The kind and the image of animated image `index`.
    ///
    /// # Safety
    /// `element` is a live `SerializedSkinImpl`.
    pub unsafe fn animated(&self, element: usize, index: u32) -> Result<(u32, AmmlImageView), AmmlStatus> {
        let layout = self.animated.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the caller guarantees a live skin.
        let count = unsafe { Self::animated_len(&layout, element) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let index = index as usize;
        if index >= count {
            return Err(AmmlStatus::INVALID_ARGUMENT);
        }
        // SAFETY: the vector is a member of the live skin and `index` is inside it.
        let [begin, _] =
            unsafe { ObjectRead::value::<[usize; 2]>(element, layout.list) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let base = begin.checked_add(index * layout.stride).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the element lies inside the live vector, and its members lie inside the element.
        let kind = unsafe { ObjectRead::value::<u32>(base, layout.kind) }.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: as above.
        let image = unsafe { ObjectRead::value::<MceImageLayout>(base, layout.image) };
        Ok((kind, Self::view(image, &ANIMATED_LOGGED, "animated")?))
    }

    /// Whether the skin is a persona skin.
    ///
    /// # Safety
    /// `element` is a live `SerializedSkinImpl`.
    pub unsafe fn is_persona(&self, element: usize) -> Result<bool, AmmlStatus> {
        let details = self.details.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the flag is a member of the live skin.
        Self::flag(unsafe { ObjectRead::value::<u8>(element, details.persona) })
    }

    /// Whether the skin is a premium skin.
    ///
    /// # Safety
    /// `element` is a live `SerializedSkinImpl`.
    pub unsafe fn is_premium(&self, element: usize) -> Result<bool, AmmlStatus> {
        let details = self.details.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the flag is a member of the live skin.
        Self::flag(unsafe { ObjectRead::value::<u8>(element, details.premium) })
    }

    /// A view of the resource patch text.
    ///
    /// # Safety
    /// `element` is a live `SerializedSkinImpl`.
    pub unsafe fn resource_patch(&self, element: usize) -> Result<AmmlStr, AmmlStatus> {
        let details = self.details.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the text is a `std::string` member of the live skin.
        unsafe { Self::text(element, details.resource_patch) }
    }

    /// A view of the default geometry name.
    ///
    /// # Safety
    /// `element` is a live `SerializedSkinImpl`.
    pub unsafe fn geometry_name(&self, element: usize) -> Result<AmmlStr, AmmlStatus> {
        let geometry = self.geometry.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the name is a `std::string` member of the live skin.
        unsafe { Self::text(element, geometry.name) }
    }

    /// The geometry JSON as text the game builds with `Json::Value::toStyledString`: the string is built on the game
    /// heap and the caller owns it. A text above `MAX_JSON` or one that is not UTF-8 is freed on `heap` and gives
    /// `NOT_AVAILABLE`.
    ///
    /// # Safety
    /// `element` is a live `SerializedSkinImpl`; `heap` is the game heap.
    pub unsafe fn geometry_json(&self, element: usize, heap: &MsvcHeap) -> Result<Box<MsvcString>, AmmlStatus> {
        let geometry = self.geometry.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let value = element.checked_add(geometry.data).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        let mut out = Box::new(MaybeUninit::<MsvcString>::zeroed());
        // SAFETY: `to_styled_string` is the resolved `Json::Value::toStyledString`: this in rcx, the address of an
        // uninitialised std::string in rdx, which it constructs and returns.
        unsafe {
            std::mem::transmute::<usize, ToStyledStringFn>(geometry.to_styled_string)(value, out.as_mut_ptr());
        }
        // SAFETY: the call built the string in `out`.
        let mut text = unsafe { Box::from_raw(Box::into_raw(out).cast::<MsvcString>()) };
        if text.as_bytes().len() > Self::MAX_JSON || std::str::from_utf8(text.as_bytes()).is_err() {
            // SAFETY: the game built the string with its own allocator, which `heap` calls; it is not used again.
            unsafe { text.release(heap) };
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        Ok(text)
    }

    /// A view of the `std::string` at `base + offset`: the size is at +10h, the capacity at +18h, the bytes are inline
    /// up to 15 characters and behind the pointer in the first word above that. `NOT_AVAILABLE` for a string that does
    /// not look sane or is not UTF-8.
    ///
    /// # Safety
    /// A live `std::string` is at `base + offset`.
    pub unsafe fn text(base: usize, offset: usize) -> Result<AmmlStr, AmmlStatus> {
        let address = base.checked_add(offset).ok_or(AmmlStatus::NOT_AVAILABLE)?;
        // SAFETY: the caller guarantees a live `std::string` at `address`.
        let (size, capacity) = unsafe {
            (
                ObjectRead::value::<usize>(address, Self::STRING_SIZE_AT),
                ObjectRead::value::<usize>(address, Self::STRING_CAPACITY_AT),
            )
        };
        let (Some(size), Some(capacity)) = (size, capacity) else { return Err(AmmlStatus::NOT_AVAILABLE) };
        if size > capacity || size > Self::MAX_TEXT {
            return Err(AmmlStatus::NOT_AVAILABLE);
        }
        if size == 0 {
            return Ok(AmmlStr::EMPTY);
        }
        let data = if capacity <= Self::INLINE {
            address
        } else {
            // SAFETY: a heap string keeps its data pointer in the first word.
            unsafe { ObjectRead::pointer(address, 0) }.ok_or(AmmlStatus::NOT_AVAILABLE)?
        };
        // SAFETY: the live string holds `size` readable bytes at `data`, inline or on the heap.
        let bytes = unsafe { std::slice::from_raw_parts(std::ptr::with_exposed_provenance::<u8>(data), size) };
        AmmlStr::from_bytes(bytes).map_err(|_| AmmlStatus::NOT_AVAILABLE)
    }

    fn flag(value: Option<u8>) -> Result<bool, AmmlStatus> {
        value.map(|flag| flag != 0).ok_or(AmmlStatus::NOT_AVAILABLE)
    }

    // The number of elements of the animated vector; `None` for a vector that does not look sane.
    unsafe fn animated_len(layout: &SkinAnimatedLayout, element: usize) -> Option<usize> {
        // SAFETY: the vector (begin, end) is a member of the live skin.
        let [begin, end] = unsafe { ObjectRead::value::<[usize; 2]>(element, layout.list) }?;
        if begin == 0 {
            return Some(0);
        }
        let bytes = end.checked_sub(begin)?;
        (bytes % layout.stride == 0).then(|| (bytes / layout.stride).min(SkinAnimatedLayout::MAX_IMAGES))
    }

    // The view of an image the game owns. The first read of each kind is logged with the values found.
    fn view(
        image: Option<MceImageLayout>,
        logged: &AtomicBool,
        kind: &'static str,
    ) -> Result<AmmlImageView, AmmlStatus> {
        let image = image.ok_or(AmmlStatus::NOT_AVAILABLE)?;
        if !logged.swap(true, Ordering::AcqRel) {
            curia::info!("first skin image read", {
                kind: kind,
                width: image.width,
                height: image.height,
                format: image.format,
                blob_size: image.blob_size
            });
        }
        image.view().ok_or(AmmlStatus::NOT_AVAILABLE)
    }
}
