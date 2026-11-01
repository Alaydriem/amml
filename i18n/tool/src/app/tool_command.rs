use std::path::PathBuf;

use crate::error::ToolError;

/// One run of the tool, parsed from the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolCommand {
    /// Compile every `<locale>.po` in `po_dir` that ships into `out_dir/<locale>.json`.
    Build { po_dir: PathBuf, out_dir: PathBuf },
    /// Write the `en_XA` pack for `pot` to `out`.
    Pseudo { pot: PathBuf, out: PathBuf },
    /// Print the coverage table of every .po in `po_dir` against `pot`.
    Report { pot: PathBuf, po_dir: PathBuf },
    /// Fail when `pot` differs from the freshly written `fresh`.
    CheckPot { pot: PathBuf, fresh: PathBuf },
}

impl ToolCommand {
    pub const USAGE: &'static str = "amml-i18n-tool build <po-dir> <out-dir> | pseudo <pot> <out-json> | \
         report <pot> <po-dir> | check-pot <pot> <fresh-pot>";

    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, ToolError> {
        let args: Vec<String> = args.into_iter().collect();
        let [command, first, second] = args.as_slice() else {
            return Err(ToolError::Usage(Self::USAGE.to_owned()));
        };
        let (first, second) = (PathBuf::from(first), PathBuf::from(second));
        match command.as_str() {
            "build" => Ok(Self::Build { po_dir: first, out_dir: second }),
            "pseudo" => Ok(Self::Pseudo { pot: first, out: second }),
            "report" => Ok(Self::Report { pot: first, po_dir: second }),
            "check-pot" => Ok(Self::CheckPot { pot: first, fresh: second }),
            _ => Err(ToolError::Usage(Self::USAGE.to_owned())),
        }
    }
}
