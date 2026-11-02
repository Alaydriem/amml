use crate::structs::PowerShellText;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::path::Path;

// The PowerShell that copies the game exe with the game's package identity, so Windows serves the locked file.
//
// The script holds no path. The source and destination folders travel in the environment variables
// `AMML_SRC` and `AMML_DEST`, so no quote character in a user name can end a string early. The script starts a
// second PowerShell inside the package with `Invoke-CommandInDesktopPackage`; that one runs robocopy and writes
// robocopy's exit code to `robocopy.exit` in the destination folder, which is how the caller learns the copy is
// finished and whether it worked.
pub struct PackageContextCommand {
    family: String,
    app_id: String,
    src_dir: String,
    dest_dir: String,
}

impl PackageContextCommand {
    pub const EXE_NAME: &'static str = "Minecraft.Windows.exe";
    pub const MARKER_NAME: &'static str = "robocopy.exit";
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

    pub fn script(&self) -> String {
        // the inner script gets its three paths as Base64 text
        let inner = "$inner = '$ErrorActionPreference = ''Stop''; ' + '$text = [Text.Encoding]::UTF8; ' + \
             '$src = $text.GetString([Convert]::FromBase64String(''' + (Pack $src) + ''')); ' + \
             '$dest = $text.GetString([Convert]::FromBase64String(''' + (Pack $dest) + ''')); ' + \
             '$robocopy = $text.GetString([Convert]::FromBase64String(''' + (Pack $robocopy) + ''')); ' + \
             '$marker = Join-Path $dest ''robocopy.exit''; ' + \
             'try { & $robocopy $src $dest Minecraft.Windows.exe /COPY:DT /R:1 /W:1 /NP | Out-Null; ' + \
             'Set-Content -LiteralPath $marker -Value $LASTEXITCODE -Encoding ASCII } ' + \
             'catch { Set-Content -LiteralPath $marker -Value (''error '' + $_.Exception.Message) -Encoding UTF8 }'";
        let lines = [
            "$ErrorActionPreference = 'Stop'".to_string(),
            format!("$src = $env:{}", Self::SRC_VARIABLE),
            format!("$dest = $env:{}", Self::DEST_VARIABLE),
            "$system = Join-Path $env:SystemRoot 'System32'".to_string(),
            "$robocopy = Join-Path $system 'robocopy.exe'".to_string(),
            "$shell = Join-Path $system 'WindowsPowerShell\\v1.0\\powershell.exe'".to_string(),
            "$text = [Text.Encoding]::UTF8".to_string(),
            "function Pack($value) { [Convert]::ToBase64String($text.GetBytes($value)) }".to_string(),
            inner.to_string(),
            "$encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($inner))".to_string(),
            "$arguments = '-NoProfile -NonInteractive -ExecutionPolicy Bypass -EncodedCommand ' + $encoded".to_string(),
            format!(
                "Invoke-CommandInDesktopPackage -PackageFamilyName {} -AppId {} -Command $shell -Args $arguments -ErrorAction Stop",
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
