/// One game shape of an event: all its entries must resolve for the shape to hold. Its optional entries
/// add to the shape when they resolve and are never named as missing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdapterShape {
    pub name: &'static str,
    pub entries: &'static [&'static str],
    pub optional: &'static [&'static str],
}

impl AdapterShape {
    /// The shape list of an event with one game shape.
    pub const fn single(name: &'static str, entries: &'static [&'static str]) -> [Self; 1] {
        [Self { name, entries, optional: &[] }]
    }
}
