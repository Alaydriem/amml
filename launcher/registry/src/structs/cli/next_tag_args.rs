use clap::Args;

#[derive(Debug, Args)]
pub struct NextTagArgs {
    /// The UTC date, YYYY-MM-DD
    #[arg(long)]
    pub date: String,
    /// Tags that already exist
    #[arg(long, num_args = 0..)]
    pub existing: Vec<String>,
}
