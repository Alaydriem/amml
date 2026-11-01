use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use pelite::pe64::{Pe, PeFile};
use xxhash_rust::xxh3::xxh3_64;

use crate::errors::DataError;
use crate::structs::BuildId;

const SCN_MEM_EXECUTE: u32 = 0x2000_0000;
// large enough for the DOS stub, the NT headers and the section table of any real executable
const HEADER_WINDOW: u64 = 1 << 20;

/// Computes the `BuildId` of a game executable from the file on disk.
///
/// The id is the `TimeDateStamp` of the file header plus the xxh3 of the raw bytes of the first
/// executable section. Both survive a patch that adds sections or rewrites the import table.
/// The mapped image must never be used instead, because its section bytes differ from the file.
pub struct BuildIdReader;

impl BuildIdReader {
    /// Reads the PE headers and the first executable section from `path`.
    pub fn from_path(path: &Path) -> Result<BuildId, DataError> {
        let io = |source| DataError::Io { path: path.to_path_buf(), source };
        let mut file = File::open(path).map_err(io)?;
        let length = file.metadata().map_err(io)?.len();
        let mut headers = Vec::new();
        file.by_ref().take(HEADER_WINDOW.min(length)).read_to_end(&mut headers).map_err(io)?;
        let (timestamp, offset, size) = Self::locate(path, &headers)?;
        file.seek(SeekFrom::Start(u64::from(offset))).map_err(io)?;
        let mut text = Vec::new();
        file.take(u64::from(size)).read_to_end(&mut text).map_err(io)?;
        if text.len() != size as usize {
            return Err(Self::truncated(path));
        }
        Ok(BuildId::new(timestamp, xxh3_64(&text)))
    }

    /// Same value as `from_path`, for a file already in memory.
    pub fn from_bytes(bytes: &[u8]) -> Result<BuildId, DataError> {
        let path = Path::new("");
        let (timestamp, offset, size) = Self::locate(path, bytes)?;
        let start = offset as usize;
        let text = start
            .checked_add(size as usize)
            .and_then(|end| bytes.get(start..end))
            .ok_or_else(|| Self::truncated(path))?;
        Ok(BuildId::new(timestamp, xxh3_64(text)))
    }

    fn truncated(path: &Path) -> DataError {
        DataError::NotPe {
            path: path.to_path_buf(),
            message: "the executable section runs past the end of the file".to_string(),
        }
    }

    // (TimeDateStamp, file offset, raw size) of the first executable section
    fn locate(path: &Path, headers: &[u8]) -> Result<(u32, u32, u32), DataError> {
        let not_pe = |message: String| DataError::NotPe { path: path.to_path_buf(), message };
        let file = PeFile::from_bytes(headers).map_err(|error| not_pe(error.to_string()))?;
        let section = file
            .section_headers()
            .iter()
            .find(|header| header.Characteristics & SCN_MEM_EXECUTE != 0)
            .ok_or_else(|| not_pe("no executable section".to_string()))?;
        if section.PointerToRawData == 0 {
            return Err(not_pe("the executable section has no file data".to_string()));
        }
        Ok((file.file_header().TimeDateStamp, section.PointerToRawData, section.SizeOfRawData))
    }
}
