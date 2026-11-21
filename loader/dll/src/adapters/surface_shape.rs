use amml_data::AdapterCatalog;

/// How a build asks a block whether it gives full support on a face.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceShape {
    /// 26.63: `Block::canProvideFullSupport(face)`.
    Support,
    /// 70.x: `Block::canProvideFullSupport(face, flag)`; the loader gives flag 0.
    SupportFlag,
}

impl SurfaceShape {
    /// The shape the resolved entries give: `Support` when `Block.canProvideFullSupport` resolved, `SupportFlag` when
    /// `Block.canProvideFullSupportWithFlag` did, `None` when the row is not complete in either. `resolved` says
    /// whether an entry name resolved.
    pub fn pick(resolved: &impl Fn(&str) -> bool) -> Option<Self> {
        AdapterCatalog::availability(resolved)
            .into_iter()
            .find(|row| row.id == "world.scan_surfaces")
            .and_then(|row| row.shape)
            .and_then(Self::named)
    }

    // The shape a catalog shape name stands for.
    fn named(name: &str) -> Option<Self> {
        match name {
            AdapterCatalog::SURFACE_SUPPORT_SHAPE => Some(Self::Support),
            AdapterCatalog::SURFACE_SUPPORT_FLAG_SHAPE => Some(Self::SupportFlag),
            _ => None,
        }
    }
}
