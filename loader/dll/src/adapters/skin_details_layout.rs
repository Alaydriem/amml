/// Where the details of a `SerializedSkinImpl` are: the persona flag, the premium flag and the resource patch text. From
/// the entries `SerializedSkinImpl.mIsPersona`, `.mIsPremium` and `.mResourcePatch`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkinDetailsLayout {
    pub persona: usize,
    pub premium: usize,
    pub resource_patch: usize,
}
