pub struct PowerShellText;

impl PowerShellText {
    // A PowerShell single-quoted string. PowerShell reads the typographic single quotes as quotes too, so each of
    // them is doubled like the plain one.
    pub fn quote(text: &str) -> String {
        let mut quoted = String::from("'");
        for character in text.chars() {
            quoted.push(character);
            if matches!(character, '\'' | '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}') {
                quoted.push(character);
            }
        }
        quoted.push('\'');
        quoted
    }
}
