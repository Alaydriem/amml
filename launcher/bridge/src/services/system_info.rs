use std::os::windows::process::CommandExt;
use std::process::Command;

// Facts about the machine for the bug report.
pub struct SystemInfo;

impl SystemInfo {
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    // `10.0.26300.7171`, from the text `cmd /c ver` prints; none when the text has no version
    pub fn windows_version() -> Option<String> {
        let output =
            Command::new("cmd.exe").args(["/C", "ver"]).creation_flags(Self::CREATE_NO_WINDOW).output().ok()?;
        Self::version_in(&String::from_utf8_lossy(&output.stdout))
    }

    // the dotted number after `Version` in `Microsoft Windows [Version 10.0.26300.7171]`
    pub fn version_in(text: &str) -> Option<String> {
        // a localised Windows may print "version" in lower case
        let at = text.to_ascii_lowercase().find("version")?;
        let after = &text[at + "version".len()..];
        let number: String = after
            .trim_start()
            .chars()
            .take_while(|character| character.is_ascii_digit() || *character == '.')
            .collect();
        if number.is_empty() { None } else { Some(number) }
    }
}
