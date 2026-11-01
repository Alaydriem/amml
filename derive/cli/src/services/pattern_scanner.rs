use memchr::memmem;

use crate::structs::{Image, Pattern};

pub struct PatternScanner;

impl PatternScanner {
    pub fn find_all(image: &Image, pattern: &Pattern) -> Vec<u32> {
        Self::find_in(image, pattern, (0, u32::MAX))
    }

    /// Matches whose start RVA lies in `range` (start inclusive, end exclusive), ascending.
    /// Only the part of each section that can hold such a start is searched.
    pub fn find_in(image: &Image, pattern: &Pattern, range: (u32, u32)) -> Vec<u32> {
        let (anchor_at, anchor) = pattern.anchor();
        let finder = memmem::Finder::new(&anchor);
        let mut found = Vec::new();
        for section in image.sections().iter().filter(|s| s.executable) {
            if range.1 <= section.rva {
                continue;
            }
            // anchor hits for starts in [range.0, range.1), as offsets into the section
            let first_start = range.0.saturating_sub(section.rva) as usize;
            let end_start = ((range.1 - section.rva) as usize).min(section.data.len());
            let window_start = first_start.saturating_add(anchor_at);
            let window_end =
                (end_start + anchor.len()).saturating_sub(1).saturating_add(anchor_at).min(section.data.len());
            if window_start >= window_end {
                continue;
            }
            let window = &section.data[window_start..window_end];
            let mut pos = 0;
            // overlapping anchor hits count, so advance one byte past each hit
            while let Some(found_at) = finder.find(&window[pos..]) {
                let hit = window_start + pos + found_at;
                pos += found_at + 1;
                let Some(start) = hit.checked_sub(anchor_at) else { continue };
                if !pattern.matches_at(&section.data, start) {
                    continue;
                }
                let Ok(offset) = u32::try_from(start) else { continue };
                let Some(rva) = section.rva.checked_add(offset) else { continue };
                if rva >= range.0 && rva < range.1 {
                    found.push(rva);
                }
            }
        }
        found.sort_unstable();
        found
    }
}
