/// The `CheckSum` of the PE optional header, as `CheckSumMappedFile` computes it.
pub struct PeChecksum;

impl PeChecksum {
    /// Sums `bytes` as 16-bit words with the four bytes at `checksum_offset` counted as zero, folds
    /// the carries and adds the file length.
    pub fn compute(bytes: &[u8], checksum_offset: usize) -> u32 {
        let mut sum: u64 = 0;
        for (index, pair) in bytes.chunks(2).enumerate() {
            let at = index * 2;
            if (checksum_offset..checksum_offset + 4).contains(&at) {
                continue;
            }
            let word = u64::from(pair[0]) | pair.get(1).map_or(0, |high| u64::from(*high) << 8);
            sum += word;
            sum = (sum & 0xffff) + (sum >> 16);
        }
        sum = (sum & 0xffff) + (sum >> 16);
        sum += bytes.len() as u64;
        sum as u32
    }
}
