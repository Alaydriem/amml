/// A function of the new build that looks like the old function.
#[derive(Clone, Debug, PartialEq)]
pub struct MatchCandidate {
    pub rva: u32,
    /// The Jaccard index of the anchors of both functions, from 0 to 1.
    pub score: f64,
    pub shared_strings: usize,
    pub shared_constants: usize,
    pub shared_imports: usize,
    pub size: u32,
}

impl MatchCandidate {
    /// `0x<rva>  score <score>  strings <n>  imports <n>  constants <n>  size <n>`
    pub fn line(&self) -> String {
        format!(
            "{:#x}  score {:.3}  strings {}  imports {}  constants {}  size {}",
            self.rva, self.score, self.shared_strings, self.shared_imports, self.shared_constants, self.size
        )
    }
}
