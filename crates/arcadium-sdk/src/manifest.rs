use serde::Deserialize;

use crate::GameMetadata;

#[derive(Debug, Deserialize)]
pub struct AdmManifest {
    pub game: AdmGameSection,
    pub arcadium: AdmArcadiumSection,
}

#[derive(Debug, Deserialize)]
pub struct AdmGameSection {
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct AdmArcadiumSection {
    pub sdk: u32,
    pub entry: String,
}

impl AdmManifest {
    pub fn metadata(&self) -> GameMetadata {
        GameMetadata {
            id: self.game.id.clone(),
            name: self.game.name.clone(),
            author: self.game.author.clone(),
            version: self.game.version.clone(),
            description: self.game.description.clone(),
        }
    }
}
