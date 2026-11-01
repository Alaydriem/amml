/// The ruler's time labels: digits and colons, the same in every locale. Minutes and seconds (`00:29`), with hours
/// for a replay of an hour or more (`1:05:00`), and hundredths when the ticks are less than a second apart
/// (`00:29.50`).
pub(crate) struct Timecode;

impl Timecode {
    pub(crate) fn label(seconds: f64, step: f64, long: bool) -> String {
        let hundredths = (seconds.max(0.0) * 100.0).round() as u64;
        let whole = hundredths / 100;
        let (hours, minutes, secs) = (whole / 3600, whole / 60 % 60, whole % 60);
        let base =
            if long { format!("{hours}:{minutes:02}:{secs:02}") } else { format!("{:02}:{secs:02}", whole / 60) };
        if step < 1.0 { format!("{base}.{:02}", hundredths % 100) } else { base }
    }

    /// The widest label the ruler can show for a replay `long` or not, at a step under a second or not.
    pub(crate) fn widest(step: f64, long: bool) -> String {
        Self::label(if long { 35_999.99 } else { 5_999.99 }, step, long).replace(|c: char| c.is_ascii_digit(), "0")
    }
}
