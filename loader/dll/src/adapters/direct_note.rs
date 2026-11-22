use std::net::IpAddr;

/// The direct join `startExternalNetworkWorld` raised: its address (the host, or the unresolved URL when the host is
/// empty), its port and the frame it was seen at. The game builds the info again from the parsed address, so the text
/// it passes on can differ (an IPv6 address in another form); an address literal is compared as an address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectNote {
    address: String,
    parsed: Option<IpAddr>,
    port: u16,
    frame: u64,
}

impl DirectNote {
    pub fn new(address: String, port: u16, frame: u64) -> Self {
        let parsed = address.parse().ok();
        Self { address, parsed, port, frame }
    }

    /// Whether a join to `host` or `url` at `port` is this direct join passed on.
    pub fn matches(&self, host: &str, url: &str, port: u16) -> bool {
        if port != self.port || self.address.is_empty() {
            return false;
        }
        [host, url]
            .iter()
            .any(|text| *text == self.address || (self.parsed.is_some() && text.parse::<IpAddr>().ok() == self.parsed))
    }

    /// Whether more than `frames` frames passed since the note at frame `now`.
    pub fn expired(&self, now: u64, frames: u64) -> bool {
        now.saturating_sub(self.frame) > frames
    }
}
