use crate::errors::RegistryError;

/// Names registry release tags, `YYYY.MM.DD-N`, and never reuses one.
pub struct TagNamer;

impl TagNamer {
    pub fn next(date: &str, existing: &[String]) -> Result<String, RegistryError> {
        let parts: Vec<&str> = date.split('-').collect();
        let shape = parts.len() == 3
            && [4, 2, 2]
                .iter()
                .zip(&parts)
                .all(|(width, part)| part.len() == *width && part.bytes().all(|b| b.is_ascii_digit()));
        if !shape {
            return Err(RegistryError::Usage(format!("date `{date}` is not YYYY-MM-DD")));
        }
        let prefix = format!("{}-", parts.join("."));
        let taken: Vec<u32> = existing
            .iter()
            .filter_map(|tag| tag.strip_prefix(&prefix))
            .filter_map(|count| count.parse::<u32>().ok())
            .collect();
        let mut number = 1;
        while taken.contains(&number) {
            number += 1;
        }
        Ok(format!("{prefix}{number}"))
    }
}
