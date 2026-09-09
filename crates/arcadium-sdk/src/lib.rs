//defines what kind of metadatas a game should have.
#[derive(Debug, Clone)]
pub struct GameMetadata {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
}
