use std::path::PathBuf;

use arcadium_sdk::GameMetadata;

#[derive(Debug, Clone)]
pub struct InstalledGame {
    pub metadata: GameMetadata,
    pub source_path: PathBuf,
    pub sdk_version: u32,
    pub entry: String,
}

impl InstalledGame {
    pub fn new(
        metadata: GameMetadata,
        source_path: PathBuf,
        sdk_version: u32,
        entry: String,
    ) -> Self {
        Self {
            metadata,
            source_path,
            sdk_version,
            entry,
        }
    }
}
