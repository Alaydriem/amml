use amml_gdk::RobocopyLog;

#[test]
fn an_access_denied_copy_gives_the_error_line_and_its_text() {
    let log = "\u{feff}\r\n  Started : Monday\r\n\t    New File  \t\t 296.8 m\tMinecraft.Windows.exe\r\n\
        2026/11/02 17:30:09 ERROR 5 (0x00000005) Copying File C:\\Pkg\\Minecraft.Windows.exe\r\n\
        Access is denied.\r\n\r\n   Files :         1         0\r\n";

    assert_eq!(
        RobocopyLog::errors_in(log),
        vec![
            "2026/11/02 17:30:09 ERROR 5 (0x00000005) Copying File C:\\Pkg\\Minecraft.Windows.exe".to_string(),
            "Access is denied.".to_string(),
        ]
    );
}

#[test]
fn a_translated_error_line_is_found_by_its_code() {
    let log = "2026/11/02 17:30:09 FEHLER 5 (0x00000005) Kopieren der Datei C:\\Pkg\\a.exe\nZugriff verweigert\n";

    assert_eq!(RobocopyLog::errors_in(log).len(), 2);
}

#[test]
fn a_clean_log_and_text_that_only_looks_like_a_code_give_nothing() {
    let log = "  Started : Monday\n  Files : 1 1 0 0 0 0\n  Speed : (0x1234) and (0xZZZZZZZZ) and (0x000000051)\n";

    assert!(RobocopyLog::errors_in(log).is_empty());
}

#[test]
fn a_missing_log_gives_nothing() {
    let dir = tempfile::tempdir().unwrap();

    assert!(RobocopyLog::errors(&dir.path().join("robocopy.log")).is_empty());
}

#[test]
fn a_utf16_log_file_is_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("robocopy.log");
    let text = "\u{feff}x ERROR 5 (0x00000005) Copying File a\r\nAccess is denied.\r\n";
    let bytes: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
    std::fs::write(&path, bytes).unwrap();

    assert_eq!(RobocopyLog::errors(&path), vec!["x ERROR 5 (0x00000005) Copying File a", "Access is denied."]);
}
