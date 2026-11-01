#[derive(Clone, Debug)]
pub struct FunctionTable {
    // (start, end, primary start), sorted by start; fragments carry their parent's start
    ranges: Vec<(u32, u32, u32)>,
}

impl FunctionTable {
    pub fn new(ranges: Vec<(u32, u32)>) -> Self {
        Self::with_fragments(ranges, Vec::new())
    }

    /// `primaries` are `(start, end)`; `fragments` are chained unwind entries as
    /// `(start, end, primary start)`.
    pub fn with_fragments(primaries: Vec<(u32, u32)>, fragments: Vec<(u32, u32, u32)>) -> Self {
        let mut ranges: Vec<_> = primaries.into_iter().map(|(s, e)| (s, e, s)).collect();
        ranges.extend(fragments);
        ranges.sort_unstable();
        Self { ranges }
    }

    /// The `(start, end)` range holding `rva`, with `end` exclusive. For an address in a
    /// chained fragment, `start` is the start of the primary function and `end` is the end of
    /// the fragment.
    pub fn containing(&self, rva: u32) -> Option<(u32, u32)> {
        let after = self.ranges.partition_point(|&(start, _, _)| start <= rva);
        let &(_, end, primary) = self.ranges.get(after.checked_sub(1)?)?;
        (rva < end).then_some((primary, end))
    }

    /// The start of every primary function, ascending.
    pub fn primaries(&self) -> Vec<u32> {
        self.ranges.iter().filter(|r| r.0 == r.2).map(|r| r.0).collect()
    }

    /// Every primary function start with all of its ranges, built in one pass.
    pub fn grouped(&self) -> Vec<(u32, Vec<(u32, u32)>)> {
        let mut groups: std::collections::BTreeMap<u32, Vec<(u32, u32)>> = std::collections::BTreeMap::new();
        for &(start, end, primary) in &self.ranges {
            groups.entry(primary).or_default().push((start, end));
        }
        groups.into_iter().collect()
    }

    /// True only for the start of a primary function.
    pub fn is_start(&self, rva: u32) -> bool {
        self.ranges.binary_search_by_key(&rva, |&(start, _, _)| start).is_ok_and(|i| self.ranges[i].2 == rva)
    }

    /// Every range of the function whose primary starts at `primary`: its own range and its
    /// fragments, ascending by start.
    pub fn ranges_of(&self, primary: u32) -> Vec<(u32, u32)> {
        self.ranges.iter().filter(|r| r.2 == primary).map(|&(start, end, _)| (start, end)).collect()
    }
}
