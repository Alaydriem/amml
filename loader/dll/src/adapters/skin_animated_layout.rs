use crate::MceImageLayout;

/// Where the animated images of a `SerializedSkinImpl` are: the vector member, the size of one element and the type
/// and the image inside an element. From the entries `SerializedSkinImpl.mSkinAnimatedImages`, `AnimatedImageData.size`,
/// `.mType` and `.mImage`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkinAnimatedLayout {
    pub list: usize,
    pub stride: usize,
    pub kind: usize,
    pub image: usize,
}

impl SkinAnimatedLayout {
    /// Most animated images read of one skin; more are not counted.
    pub const MAX_IMAGES: usize = 16;

    /// The layout, or `None` when the type or the image of an element does not fit inside the stride.
    pub fn new(list: usize, stride: usize, kind: usize, image: usize) -> Option<Self> {
        let kind_end = kind.checked_add(size_of::<u32>())?;
        let image_end = image.checked_add(size_of::<MceImageLayout>())?;
        (stride != 0 && kind_end <= stride && image_end <= stride).then_some(Self { list, stride, kind, image })
    }
}
