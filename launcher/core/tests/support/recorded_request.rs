#[derive(Debug, Clone)]
pub struct RecordedRequest {
    pub method: String,
    pub path: String,
    pub if_none_match: Option<String>,
    pub range: Option<String>,
}
