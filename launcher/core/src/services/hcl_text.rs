// Text written into HCL files.
pub struct HclText;

impl HclText {
    // a quoted HCL string; backslashes, quotes and template markers are escaped
    pub fn quote(text: &str) -> String {
        let mut quoted = String::with_capacity(text.len() + 2);
        quoted.push('"');
        let mut chars = text.chars().peekable();
        while let Some(character) = chars.next() {
            match character {
                '\\' => quoted.push_str("\\\\"),
                '"' => quoted.push_str("\\\""),
                '\n' => quoted.push_str("\\n"),
                '\r' => quoted.push_str("\\r"),
                '\t' => quoted.push_str("\\t"),
                '$' if chars.peek() == Some(&'{') => quoted.push_str("$$"),
                '%' if chars.peek() == Some(&'{') => quoted.push_str("%%"),
                other => quoted.push(other),
            }
        }
        quoted.push('"');
        quoted
    }
}
