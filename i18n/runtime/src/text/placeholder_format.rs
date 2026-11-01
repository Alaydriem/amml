use crate::localize::Localizer;

use super::Args;

/// Fills `{name}` placeholders in a message.
pub struct PlaceholderFormat;

impl PlaceholderFormat {
    /// Replaces each `{name}` that `args` has a value for. A placeholder with no value stays
    /// as written, so the missing argument is visible on screen.
    pub fn apply(template: &str, args: &Args, localizer: &Localizer) -> String {
        let mut out = String::with_capacity(template.len());
        let mut rest = template;
        while let Some(open) = rest.find('{') {
            out.push_str(&rest[..open]);
            let after = &rest[open + 1..];
            match after.find('}').and_then(|close| args.get(&after[..close]).map(|value| (close, value))) {
                Some((close, value)) => {
                    out.push_str(&value.render(localizer));
                    rest = &after[close + 1..];
                }
                None => {
                    out.push('{');
                    rest = after;
                }
            }
        }
        out.push_str(rest);
        out
    }
}
