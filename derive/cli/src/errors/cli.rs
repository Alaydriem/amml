use std::path::PathBuf;

use amml_data::DataError;

use super::{DeriveError, StepError};

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error(transparent)]
    Data(#[from] DataError),
    #[error(transparent)]
    Derive(#[from] DeriveError),
    #[error(transparent)]
    Step(#[from] StepError),
    #[error(transparent)]
    Store(DataError),
    #[error("--count must be at least 1")]
    ZeroCount,
    #[error("a vtable needs --rva or --class")]
    NoVtableSource,
    #[error("xrefs needs exactly one of --rva or --string")]
    XrefsSource,
    #[error("the operation needs --exe <path>")]
    MissingExe,
    #[error("`{0}` is not a hex address")]
    BadAddress(String),
    #[error("rva {0:#x} is outside every section")]
    AddressUnmapped(u32),
    #[error("cannot write {}: {source}", path.display())]
    Write { path: PathBuf, source: std::io::Error },
}

impl CliError {
    // 2 for input the run cannot use, 4 for output it could not write
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Data(_)
            | Self::Derive(_)
            | Self::Step(_)
            | Self::BadAddress(_)
            | Self::MissingExe
            | Self::ZeroCount
            | Self::NoVtableSource
            | Self::XrefsSource
            | Self::AddressUnmapped(_) => 2,
            Self::Store(_) | Self::Write { .. } => 4,
        }
    }
}
