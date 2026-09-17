use arcadium_sdk::{ArcadeGame, GameMetadata};

pub struct DummyGame2 {
    metadata: GameMetadata,
}

impl DummyGame2 {
    pub fn new() -> Self {
        Self {
            metadata: GameMetadata {
                id: String::from("dummy"),
                name: String::from("Dummy Game 2"),
                author: String::from("Arcadium"),
                version: String::from("0.1.0"),
                description: String::from("Reference game used to test the Arcadium SDK."),
            },
        }
    }
}

impl Default for DummyGame2 {
    fn default() -> Self {
        Self::new()
    }
}

impl ArcadeGame for DummyGame2 {
    fn metadata(&self) -> &GameMetadata {
        &self.metadata
    }
}
