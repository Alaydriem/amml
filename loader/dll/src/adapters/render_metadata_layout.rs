/// The `std::optional<dragon::RenderMetadata>` argument of `BlockActorRenderDispatcher::render`, which the loader passes
/// disengaged: zero bytes with the engaged byte at `RenderMetadata.engaged`.
///
/// The optional is 40h bytes with the engaged byte at 38h on 26.63 and 38h bytes with it at 30h on 70.26 (survey section 7,
/// rows R4 and E5); the dispatcher takes it by address and, as the callee, destroys it at its end, which for a
/// disengaged optional reads only the engaged byte (26.63 0x6a5e178 `cmp byte ptr [r14+38h],1; jne`, 70.26 0x8a22fe7
/// `cmp byte ptr [rbx+30h],1`). The loader's buffer is `SIZE` bytes, so it covers the optional of both builds; a flag
/// offset that does not leave room for the value before it inside the buffer is refused.
pub struct RenderMetadataLayout {
    engaged: usize,
}

impl RenderMetadataLayout {
    /// The size of the loader's buffer for the optional.
    pub const SIZE: usize = 0x80;

    /// The layout with the engaged byte at `engaged`; `None` for 0 or an offset at or past the end of the buffer.
    pub fn new(engaged: u64) -> Option<Self> {
        let engaged = usize::try_from(engaged).ok().filter(|engaged| *engaged > 0 && *engaged < Self::SIZE)?;
        Some(Self { engaged })
    }

    /// A disengaged optional: every byte zero, the engaged byte included.
    pub fn disengaged(&self) -> [u64; Self::SIZE / 8] {
        [0u64; Self::SIZE / 8]
    }

    /// The offset of the engaged byte.
    pub fn engaged(&self) -> usize {
        self.engaged
    }
}
