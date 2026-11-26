use amml_abi::AmmlApplyStats;

/// How a `WorldShapes::apply` went: the shape payloads handed to the game (a remove counts one, a kind change two),
/// and the time the call took in microseconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ApplyStats {
    pub delivered: u32,
    pub elapsed_us: u32,
}

impl From<AmmlApplyStats> for ApplyStats {
    fn from(stats: AmmlApplyStats) -> Self {
        Self { delivered: stats.delivered, elapsed_us: stats.elapsed_us }
    }
}
