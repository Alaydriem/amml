use amml_gdk::PackageContextCommand;
use base64::Engine;
use std::path::Path;

fn command(src: &str, dest: &str) -> PackageContextCommand {
    PackageContextCommand::new("Microsoft.MinecraftUWP_8wekyb3d8bbwe", "Game", Path::new(src), Path::new(dest))
}

fn environment(command: &PackageContextCommand) -> Vec<(String, String)> {
    command.environment().into_iter().map(|(name, value)| (name.to_string(), value)).collect()
}

#[test]
fn the_script_names_the_package_and_holds_no_path() {
    let command = command(r"C:\Program Files\WindowsApps\Pkg", r"C:\Users\o'brien\Temp\x");
    let script = command.script();
    assert!(script.contains("Invoke-CommandInDesktopPackage"));
    assert!(script.contains("-PackageFamilyName 'Microsoft.MinecraftUWP_8wekyb3d8bbwe'"));
    assert!(script.contains("-AppId 'Game'"));
    assert!(script.contains("$env:AMML_SRC") && script.contains("$env:AMML_DEST"));
    assert!(!script.contains("WindowsApps") && !script.contains("brien"));
}

#[test]
fn paths_with_any_quote_character_travel_unchanged_in_the_environment() {
    let destinations = [
        r"C:\Users\o'brien\Temp\x".to_string(),
        format!("C:{}Users{}O\u{2019}Brien{}Temp{}x", '\\', '\\', '\\', '\\'),
        format!("C:{}Users{}a\u{2018}b\u{201A}c\u{201B}d{}x", '\\', '\\', '\\'),
    ];
    for dest in destinations.iter().map(String::as_str) {
        let command = command(r"C:\Program Files\WindowsApps\Pkg", dest);
        assert!(!command.script().contains(dest));
        assert_eq!(
            environment(&command),
            vec![
                ("AMML_SRC".to_string(), r"C:\Program Files\WindowsApps\Pkg".to_string()),
                ("AMML_DEST".to_string(), dest.to_string()),
            ]
        );
    }
}

#[test]
fn typographic_quotes_in_the_family_or_app_id_are_doubled() {
    let command = PackageContextCommand::new("a\u{2019}b", "c\u{2018}d", Path::new(r"C:\a"), Path::new(r"C:\b"));
    let script = command.script();
    assert!(script.contains("-PackageFamilyName 'a\u{2019}\u{2019}b'"));
    assert!(script.contains("-AppId 'c\u{2018}\u{2018}d'"));
}

#[test]
fn trailing_backslash_is_trimmed() {
    let command = command(r"C:\a\", r"C:\b\");
    assert_eq!(
        environment(&command),
        vec![("AMML_SRC".to_string(), r"C:\a".to_string()), ("AMML_DEST".to_string(), r"C:\b".to_string())]
    );
}

#[test]
fn encoded_round_trips_utf16() {
    let command = command(r"C:\Program Files\Pkg", r"C:\Temp\x");
    let bytes = base64::engine::general_purpose::STANDARD.decode(command.encoded()).unwrap();
    let units: Vec<u16> = bytes.chunks(2).map(|pair| u16::from_le_bytes([pair[0], pair[1]])).collect();
    assert_eq!(String::from_utf16(&units).unwrap(), command.script());
}
