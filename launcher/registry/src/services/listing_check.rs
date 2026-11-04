use crate::errors::RegistryError;
use crate::structs::Listing;

/// Every rule of the mod listing, shared by the release lint and the publish workflow.
pub struct ListingCheck;

impl ListingCheck {
    pub const ICON_SIDE: u32 = 128;
    pub const ICON_MAX_BYTES: usize = 64 * 1024;
    const TITLE_MAX: usize = 40;
    const SUMMARY_MAX: usize = 120;
    const DESCRIPTION_MAX: usize = 2000;
    const AUTHORS_MAX: usize = 8;
    const AUTHOR_MAX: usize = 64;
    const TAGS_MAX: usize = 8;
    const TAG_MAX: usize = 24;
    const LICENSE_MAX: usize = 128;
    const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

    pub fn check(listing: &Listing) -> Result<(), RegistryError> {
        Self::line("title", &listing.title, 1, Self::TITLE_MAX)?;
        Self::line("summary", &listing.summary, 1, Self::SUMMARY_MAX)?;
        if let Some(description) = &listing.description {
            Self::description(description)?;
        }
        Self::authors(&listing.authors)?;
        Self::license(&listing.license)?;
        Self::url("source", &listing.source)?;
        if let Some(homepage) = &listing.homepage {
            Self::url("homepage", homepage)?;
        }
        Self::tags(&listing.tags)?;
        if let Some(icon) = &listing.icon {
            Self::icon_path(icon)?;
        }
        Ok(())
    }

    /// Checks the icon file: a PNG of 128 by 128 pixels, up to 64 KB.
    pub fn check_icon(bytes: &[u8]) -> Result<(), RegistryError> {
        if bytes.len() > Self::ICON_MAX_BYTES {
            return Err(Self::fail("icon", format!("is {} bytes, the limit is {}", bytes.len(), Self::ICON_MAX_BYTES)));
        }
        if bytes.len() < 24 || bytes[..8] != Self::PNG_SIGNATURE || &bytes[12..16] != b"IHDR" {
            return Err(Self::fail("icon", "is not a PNG file".to_string()));
        }
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        if width != Self::ICON_SIDE || height != Self::ICON_SIDE {
            return Err(Self::fail(
                "icon",
                format!("is {width} x {height} pixels, it must be {side} x {side}", side = Self::ICON_SIDE),
            ));
        }
        Ok(())
    }

    fn line(field: &'static str, text: &str, min: usize, max: usize) -> Result<(), RegistryError> {
        let count = text.chars().count();
        if count < min || count > max {
            return Err(Self::fail(field, format!("must be {min} to {max} characters, it is {count}")));
        }
        if text.chars().any(char::is_control) {
            return Err(Self::fail(field, "must be one line of plain text".to_string()));
        }
        Self::no_markup(field, text)
    }

    fn description(text: &str) -> Result<(), RegistryError> {
        let count = text.chars().count();
        if count > Self::DESCRIPTION_MAX {
            return Err(Self::fail(
                "description",
                format!("must be up to {} characters, it is {count}", Self::DESCRIPTION_MAX),
            ));
        }
        if text.chars().any(|c| c.is_control() && c != '\n') {
            return Err(Self::fail("description", "has a control character".to_string()));
        }
        Self::no_markup("description", text)
    }

    fn no_markup(field: &'static str, text: &str) -> Result<(), RegistryError> {
        let chars: Vec<char> = text.chars().collect();
        let tag = chars
            .windows(2)
            .any(|pair| pair[0] == '<' && (pair[1].is_ascii_alphabetic() || pair[1] == '/' || pair[1] == '!'));
        if tag {
            return Err(Self::fail(field, "must be plain text, without HTML".to_string()));
        }
        Ok(())
    }

    fn authors(authors: &[String]) -> Result<(), RegistryError> {
        if authors.is_empty() || authors.len() > Self::AUTHORS_MAX {
            return Err(Self::fail(
                "authors",
                format!("needs 1 to {} items, it has {}", Self::AUTHORS_MAX, authors.len()),
            ));
        }
        for author in authors {
            Self::line("authors", author, 1, Self::AUTHOR_MAX)?;
        }
        Ok(())
    }

    fn license(text: &str) -> Result<(), RegistryError> {
        let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+' | ':' | ' ' | '(' | ')');
        let count = text.chars().count();
        if count == 0 || count > Self::LICENSE_MAX || !text.chars().all(allowed) || text.trim() != text {
            return Err(Self::fail("license", "must be an SPDX license id, for example BSD-3-Clause".to_string()));
        }
        Ok(())
    }

    fn url(field: &'static str, text: &str) -> Result<(), RegistryError> {
        let has_host = text.strip_prefix("https://").is_some_and(|rest| !rest.is_empty());
        if !has_host || text.chars().any(|c| c.is_control() || c.is_whitespace()) {
            return Err(Self::fail(field, "must be an https:// URL".to_string()));
        }
        Ok(())
    }

    fn tags(tags: &[String]) -> Result<(), RegistryError> {
        if tags.len() > Self::TAGS_MAX {
            return Err(Self::fail("tags", format!("takes up to {} items, it has {}", Self::TAGS_MAX, tags.len())));
        }
        for tag in tags {
            let count = tag.chars().count();
            let shape = tag.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
            if count == 0 || count > Self::TAG_MAX || !shape {
                return Err(Self::fail("tags", format!("`{tag}` must match ^[a-z0-9-]{{1,{}}}$", Self::TAG_MAX)));
            }
        }
        Ok(())
    }

    fn icon_path(path: &str) -> Result<(), RegistryError> {
        let escapes = path.is_empty()
            || path.starts_with('/')
            || path.contains('\\')
            || path.contains(':')
            || path.split('/').any(|part| part == ".." || part.is_empty());
        if escapes || !path.to_ascii_lowercase().ends_with(".png") {
            return Err(Self::fail("icon", "must be a relative path to a .png file inside the mod folder".to_string()));
        }
        Ok(())
    }

    fn fail(field: &'static str, reason: String) -> RegistryError {
        RegistryError::Listing { field, reason }
    }
}
