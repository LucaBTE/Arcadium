use arcadium_sdk::{ArcadeGame, GameMetadata};

pub struct DummyGame {
    metadata: GameMetadata,
}

impl DummyGame {
    pub fn new() -> Self {
        Self {
            metadata: GameMetadata {
                id: String::from("dummy"),
                name: String::from("Dummy Game"),
                author: String::from("Arcadium"),
                version: String::from("0.1.0"),
                description: String::from("Reference game used to test the Arcadium SDK."),
            },
        }
    }
}

impl Default for DummyGame {
    fn default() -> Self {
        Self::new()
    }
}

impl ArcadeGame for DummyGame {
    fn metadata(&self) -> &GameMetadata {
        &self.metadata
    }
}
