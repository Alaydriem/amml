/// How much of a catalog one locale translates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coverage {
    pub total: usize,
    pub translated: usize,
    /// Rounded to one decimal place.
    pub percent: f64,
}

impl Coverage {
    /// A locale ships only at this coverage or more. A half-translated locale reads as a
    /// broken app rather than an English one.
    pub const SHIP_PERCENT: f64 = 90.0;

    pub fn new(translated: usize, total: usize) -> Self {
        let percent = if total == 0 { 100.0 } else { (translated as f64 / total as f64 * 1000.0).round() / 10.0 };
        Self { total, translated, percent }
    }

    pub fn ships(&self) -> bool {
        self.percent >= Self::SHIP_PERCENT
    }
}
