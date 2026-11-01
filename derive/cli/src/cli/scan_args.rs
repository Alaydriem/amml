use clap::Args;

#[derive(Args, Clone, Debug)]
pub struct ScanArgs {
    /// Pattern text as in a recipe: hex bytes with `?` or `??` for a wildcard
    pub pattern: String,
    /// The most matches to list
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
}
