use std::path::PathBuf;

use clap::Args;

#[derive(Args, Clone, Debug)]
pub struct ExplainArgs {
    /// The entry id, for example `ClientInstance.update`
    pub id: String,
    /// Directory with the recipe `.hcl` files
    #[arg(long, default_value = "recipes")]
    pub recipes: PathBuf,
}
