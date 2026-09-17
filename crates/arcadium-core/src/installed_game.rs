use std::path::PathBuf;

use arcadium_sdk::GameMetadata;

#[derive(Debug, Clone)]
pub struct InstalledGame {
    pub metadata: GameMetadata,
    pub source_path: PathBuf,
    pub sdk_version: u32,
}

impl InstalledGame {
    pub fn new(metadata: GameMetadata, source_path: PathBuf, sdk_version: u32) -> Self {
        Self {
            metadata,
            source_path,
            sdk_version,
        }
    }
}
