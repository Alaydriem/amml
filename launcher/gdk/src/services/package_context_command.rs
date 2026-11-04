use crate::structs::PowerShellText;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::path::Path;

// The PowerShell that copies the game exe with the game's package identity, so Windows serves the locked file.
//
// The script holds no path. The source and destination folders travel in the environment variables
// `AMML_SRC` and `AMML_DEST`, so no quote character in a user name can end a string early. The script starts
// robocopy inside the package with `Invoke-CommandInDesktopPackage`. robocopy writes a Unicode log,
// `robocopy.log`, in the destination folder; the caller reads it for errors.
pub struct PackageContextCommand {
    family: String,
    app_id: String,
    src_dir: String,
    dest_dir: String,
}

impl PackageContextCommand {
    pub const EXE_NAME: &'static str = "Minecraft.Windows.exe";
    pub const LOG_NAME: &'static str = "robocopy.log";
    pub const SRC_VARIABLE: &'static str = "AMML_SRC";
    pub const DEST_VARIABLE: &'static str = "AMML_DEST";

    pub fn new(family: &str, app_id: &str, src_dir: &Path, dest_dir: &Path) -> PackageContextCommand {
        PackageContextCommand {
            family: family.to_string(),
            app_id: app_id.to_string(),
            src_dir: Self::without_trailing_separator(src_dir),
            dest_dir: Self::without_trailing_separator(dest_dir),
        }
    }

    // the environment variables the script reads
    pub fn environment(&self) -> [(&'static str, String); 2] {
        [(Self::SRC_VARIABLE, self.src_dir.clone()), (Self::DEST_VARIABLE, self.dest_dir.clone())]
    }

    // robocopy itself must be the process the cmdlet starts: a process it starts in turn is refused the exe of an
    // older version. A Windows path cannot hold a double quote, so double quotes delimit the paths safely.
    pub fn script(&self) -> String {
        let lines = [
            "$ErrorActionPreference = 'Stop'".to_string(),
            format!("$src = $env:{}", Self::SRC_VARIABLE),
            format!("$dest = $env:{}", Self::DEST_VARIABLE),
            format!("$log = Join-Path $dest '{}'", Self::LOG_NAME),
            "$robocopy = Join-Path $env:SystemRoot 'System32\\robocopy.exe'".to_string(),
            format!(
                "$arguments = '\"' + $src + '\" \"' + $dest + '\" {} /COPY:DT /R:1 /W:1 /NP /UNILOG:\"' + $log + '\"'",
                Self::EXE_NAME
            ),
            format!(
                "Invoke-CommandInDesktopPackage -PackageFamilyName {} -AppId {} -Command $robocopy -Args $arguments -ErrorAction Stop",
                PowerShellText::quote(&self.family),
                PowerShellText::quote(&self.app_id)
            ),
        ];
        lines.join("\n")
    }

    // UTF-16LE Base64, the form `powershell.exe -EncodedCommand` reads
    pub fn encoded(&self) -> String {
        let bytes: Vec<u8> = self.script().encode_utf16().flat_map(u16::to_le_bytes).collect();
        STANDARD.encode(bytes)
    }

    // a trailing backslash would be read as an escape by a native argument parser
    fn without_trailing_separator(path: &Path) -> String {
        path.to_string_lossy().trim_end_matches(['\\', '/']).to_string()
    }
}
