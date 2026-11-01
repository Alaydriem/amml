use std::fmt;

/// A kind of design literal (rule 3 of the UI and design rules). Each one reads one line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LiteralPattern {
    /// `Color32::from_<name>(` with a number, a hex number or a string first. A colour built from values is data.
    ColourConstructor,
    /// `FontId::<name>(` with a number first.
    FontSize,
    /// `CornerRadius::same(` with a number first.
    CornerRadius,
    /// A `Shadow { ... }` struct literal.
    ShadowLiteral,
    /// `vec2(<number>, <number>)`, except on a line that declares a `const` item.
    SizeVector,
}

impl LiteralPattern {
    pub const ALL: [Self; 5] =
        [Self::ColourConstructor, Self::FontSize, Self::CornerRadius, Self::ShadowLiteral, Self::SizeVector];

    pub fn name(self) -> &'static str {
        match self {
            Self::ColourConstructor => "colour constructor",
            Self::FontSize => "font size",
            Self::CornerRadius => "corner radius",
            Self::ShadowLiteral => "shadow literal",
            Self::SizeVector => "size vector",
        }
    }

    /// True when `line` holds this literal. The caller skips comment lines.
    pub fn matches(self, line: &str) -> bool {
        match self {
            Self::ColourConstructor => Self::arguments(line, "Color32::from_", true).any(Self::starts_with_literal),
            Self::FontSize => Self::arguments(line, "FontId::", true).any(Self::starts_with_number),
            Self::CornerRadius => Self::arguments(line, "CornerRadius::same", false).any(Self::starts_with_number),
            Self::ShadowLiteral => Self::starts(line, "Shadow {").next().is_some(),
            Self::SizeVector => {
                !Self::declares_const(line) && Self::arguments(line, "vec2", false).any(Self::two_numbers)
            }
        }
    }

    /// The text after each `<call>(` in the line. With `named`, the call ends in a name (`from_rgb`, `proportional`).
    fn arguments<'a>(line: &'a str, call: &'static str, named: bool) -> impl Iterator<Item = &'a str> {
        Self::starts(line, call).filter_map(move |start| {
            let rest = &line[start + call.len()..];
            let rest = if named { rest.trim_start_matches(Self::is_name_char) } else { rest };
            rest.strip_prefix('(')
        })
    }

    /// The byte offsets of `word` where it does not continue a longer name (`myvec2(` is not `vec2(`).
    fn starts<'a>(line: &'a str, word: &'static str) -> impl Iterator<Item = usize> + 'a {
        line.match_indices(word)
            .map(|(start, _)| start)
            .filter(move |&start| !line[..start].chars().next_back().is_some_and(Self::is_name_char))
    }

    fn is_name_char(character: char) -> bool {
        character.is_ascii_alphanumeric() || character == '_'
    }

    fn starts_with_number(arguments: &str) -> bool {
        arguments.trim_start().starts_with(|character: char| character.is_ascii_digit())
    }

    fn starts_with_literal(arguments: &str) -> bool {
        Self::starts_with_number(arguments) || arguments.trim_start().starts_with('"')
    }

    fn two_numbers(arguments: &str) -> bool {
        let Some((inside, _)) = arguments.split_once(')') else {
            return false;
        };
        let parts: Vec<&str> = inside.split(',').map(str::trim).collect();
        parts.len() == 2 && parts.iter().all(|part| Self::is_number(part))
    }

    fn is_number(text: &str) -> bool {
        let digits = text.strip_prefix('-').unwrap_or(text);
        let digits = digits.strip_suffix("f32").unwrap_or(digits);
        digits.starts_with(|character: char| character.is_ascii_digit())
            && digits.chars().all(|character| character.is_ascii_digit() || character == '.' || character == '_')
    }

    fn declares_const(line: &str) -> bool {
        let item = line.trim_start();
        let item = ["pub(crate) ", "pub(super) ", "pub "]
            .iter()
            .find_map(|visibility| item.strip_prefix(visibility))
            .unwrap_or(item);
        item.starts_with("const ") && !item.starts_with("const fn ")
    }
}

impl fmt::Display for LiteralPattern {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}
