use amml_abi::AmmlLightFalloff;

/// How a held light fades with distance, measured to the block centre and rounded to whole blocks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LightFalloff {
    /// By straight-line distance: a round light.
    #[default]
    Round,
    /// By block steps (taxicab distance), as a placed torch fades: a diamond.
    Taxicab,
}

impl From<LightFalloff> for AmmlLightFalloff {
    fn from(falloff: LightFalloff) -> Self {
        match falloff {
            LightFalloff::Round => AmmlLightFalloff::ROUND,
            LightFalloff::Taxicab => AmmlLightFalloff::TAXICAB,
        }
    }
}
