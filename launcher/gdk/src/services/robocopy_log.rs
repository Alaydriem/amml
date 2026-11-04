use std::path::Path;

// Reads the error lines from a robocopy `/UNILOG` file (UTF-16LE). robocopy translates its words, so an error line
// is found by its Windows error code in the form `(0x00000005)`; the line after it holds the error text.
pub struct RobocopyLog;

impl RobocopyLog {
    pub fn errors(log: &Path) -> Vec<String> {
        let Ok(bytes) = std::fs::read(log) else {
            return Vec::new();
        };
        let units: Vec<u16> = bytes.as_chunks::<2>().0.iter().map(|pair| u16::from_le_bytes(*pair)).collect();
        Self::errors_in(&String::from_utf16_lossy(&units))
    }

    pub fn errors_in(text: &str) -> Vec<String> {
        let lines: Vec<&str> =
            text.lines().map(|line| line.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}')).collect();
        let lines: Vec<&str> = lines.into_iter().filter(|line| !line.is_empty()).collect();
        let mut found = Vec::new();
        for (index, line) in lines.iter().enumerate() {
            if Self::has_error_code(line) {
                found.push((*line).to_string());
                if let Some(next) = lines.get(index + 1) {
                    found.push((*next).to_string());
                }
            }
        }
        found
    }

    fn has_error_code(line: &str) -> bool {
        line.match_indices("(0x").any(|(start, _)| {
            let rest = &line[start + 3..];
            rest.len() > 8 && rest[..8].chars().all(|c| c.is_ascii_hexdigit()) && rest[8..].starts_with(')')
        })
    }
}
