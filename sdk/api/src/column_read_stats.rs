use amml_abi::AmmlColumnReadStats;

/// How a `LevelChunk::read_columns` call went.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ColumnReadStats {
    /// The columns that were read (a column to retry counts here too).
    pub read: u16,
    /// The columns that were asked for and not read.
    pub skipped: u16,
    /// The columns that need a retry.
    pub retry: u16,
}

impl From<AmmlColumnReadStats> for ColumnReadStats {
    fn from(stats: AmmlColumnReadStats) -> Self {
        Self { read: stats.read, skipped: stats.skipped, retry: stats.retry }
    }
}

impl From<ColumnReadStats> for AmmlColumnReadStats {
    fn from(stats: ColumnReadStats) -> Self {
        Self { read: stats.read, skipped: stats.skipped, retry: stats.retry, _reserved: 0 }
    }
}
