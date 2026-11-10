use std::path::PathBuf;

use clap::Args;

#[derive(Args, Clone, Debug)]
pub struct DeriveArgs {
    /// The game executable to read
    #[arg(long)]
    pub exe: PathBuf,
    /// Directory with the recipe `.hcl` files
    #[arg(long)]
    pub recipes: PathBuf,
    /// Directory that receives `<build>.hcl`
    #[arg(long)]
    pub out: PathBuf,
    /// Directory that receives `<build>.md`
    #[arg(long)]
    pub report: PathBuf,
    /// An earlier data file; entries whose value changed are listed in the report
    #[arg(long)]
    pub compare: Option<PathBuf>,
    /// Directory that receives `<build>/README.md` and one repair job file per failed entry
    #[arg(long)]
    pub jobs: Option<PathBuf>,
    /// Derive every entry again instead of reusing the entries of an existing `<build>.hcl` in `--out`
    #[arg(long)]
    pub full: bool,
}

impl DeriveArgs {
    pub fn new(
        exe: PathBuf,
        recipes: PathBuf,
        out: PathBuf,
        report: PathBuf,
        compare: Option<PathBuf>,
        jobs: Option<PathBuf>,
        full: bool,
    ) -> Self {
        Self { exe, recipes, out, report, compare, jobs, full }
    }
}
