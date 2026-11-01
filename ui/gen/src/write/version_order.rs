use std::cmp::Ordering;

/// rustfmt's "version sorting" (Rust style guide, style edition 2024), which it uses for `use`
/// lines: names split into runs of digits and runs of other characters; digit runs compare by
/// value, so `k2` comes before `k10`; `_` comes before every other character.
pub struct VersionOrder;

impl VersionOrder {
    pub fn compare(left: &str, right: &str) -> Ordering {
        let (mut left, mut right) = (left, right);
        // The first place where equal numbers had different leading zeros: more zeros sort first.
        let mut zeros = Ordering::Equal;
        loop {
            match (Self::chunk(left), Self::chunk(right)) {
                (None, None) => return zeros,
                (None, Some(_)) => return Ordering::Less,
                (Some(_), None) => return Ordering::Greater,
                (Some((a, a_rest)), Some((b, b_rest))) => {
                    let a_digits = a.bytes().all(|byte| byte.is_ascii_digit());
                    let b_digits = b.bytes().all(|byte| byte.is_ascii_digit());
                    let order = match (a_digits, b_digits) {
                        (true, true) => {
                            let (a_value, b_value) = (a.trim_start_matches('0'), b.trim_start_matches('0'));
                            let order = a_value.len().cmp(&b_value.len()).then_with(|| a_value.cmp(b_value));
                            if order == Ordering::Equal && zeros == Ordering::Equal {
                                zeros = b.len().cmp(&a.len());
                            }
                            order
                        }
                        _ => Self::text(a, b),
                    };
                    if order != Ordering::Equal {
                        return order;
                    }
                    (left, right) = (a_rest, b_rest);
                }
            }
        }
    }

    /// The next run of digits or of other characters, and the rest.
    fn chunk(text: &str) -> Option<(&str, &str)> {
        let first = text.bytes().next()?;
        let digits = first.is_ascii_digit();
        let end = text.bytes().position(|byte| byte.is_ascii_digit() != digits).unwrap_or(text.len());
        Some(text.split_at(end))
    }

    /// Character order with `_` before every other character; a shorter prefix comes first.
    fn text(left: &str, right: &str) -> Ordering {
        let key = |ch: char| if ch == '_' { (0, '\0') } else { (1, ch) };
        left.chars().map(key).cmp(right.chars().map(key))
    }
}
