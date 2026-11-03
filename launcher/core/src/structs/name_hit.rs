// One search result: the id, where its card is (API and index page) and how well it matched.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameHit {
    pub id: String,
    pub title: String,
    pub api: u32,
    pub page: u32,
    pub score: u32,
}
