use serde::Serialize;

use crate::errors::RegistryError;

/// The one writer of registry JSON, so every file has the same bytes for the same value.
pub struct JsonText;

impl JsonText {
    // pretty, two-space indent, one trailing newline
    pub fn pretty<T: Serialize>(value: &T) -> Result<Vec<u8>, RegistryError> {
        let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| RegistryError::Json(error.to_string()))?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    // no whitespace, one trailing newline
    pub fn compact<T: Serialize>(value: &T) -> Result<Vec<u8>, RegistryError> {
        let mut bytes = serde_json::to_vec(value).map_err(|error| RegistryError::Json(error.to_string()))?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}
