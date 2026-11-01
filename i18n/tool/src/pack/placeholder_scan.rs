use std::collections::BTreeSet;

/// Finds the `{name}` placeholders in a message.
pub struct PlaceholderScan;

impl PlaceholderScan {
    pub fn names(text: &str) -> BTreeSet<&str> {
        let mut names = BTreeSet::new();
        let mut rest = text;
        while let Some(open) = rest.find('{') {
            let after = &rest[open + 1..];
            let Some(close) = after.find('}') else { break };
            names.insert(&after[..close]);
            rest = &after[close + 1..];
        }
        names
    }
}
