use std::path::PathBuf;

use crate::error::GenError;

/// `amml-ui-gen --tokens <path> --ownership <path> --kit-out <dir> --studio-out <dir> [--check]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliArgs {
    pub tokens: PathBuf,
    pub ownership: PathBuf,
    pub kit_out: PathBuf,
    pub studio_out: PathBuf,
    /// Write nothing; list what would change.
    pub check: bool,
}

impl CliArgs {
    pub const USAGE: &'static str =
        "usage: amml-ui-gen --tokens <path> --ownership <path> --kit-out <dir> --studio-out <dir> [--check]";

    /// The arguments after the program name.
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, GenError> {
        let usage = |problem: &str| GenError::Usage { message: format!("{problem}\n{}", Self::USAGE) };
        let (mut tokens, mut ownership, mut kit_out, mut studio_out, mut check) = (None, None, None, None, false);
        let mut args = args.into_iter();
        while let Some(flag) = args.next() {
            let slot = match flag.as_str() {
                "--check" => {
                    check = true;
                    continue;
                }
                "--tokens" => &mut tokens,
                "--ownership" => &mut ownership,
                "--kit-out" => &mut kit_out,
                "--studio-out" => &mut studio_out,
                other => return Err(usage(&format!("unknown argument `{other}`"))),
            };
            let value = args.next().ok_or_else(|| usage(&format!("`{flag}` needs a value")))?;
            *slot = Some(PathBuf::from(value));
        }
        Ok(Self {
            tokens: tokens.ok_or_else(|| usage("missing --tokens"))?,
            ownership: ownership.ok_or_else(|| usage("missing --ownership"))?,
            kit_out: kit_out.ok_or_else(|| usage("missing --kit-out"))?,
            studio_out: studio_out.ok_or_else(|| usage("missing --studio-out"))?,
            check,
        })
    }
}
