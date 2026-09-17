mod game;
mod manifest;
mod metadata;

pub use game::ArcadeGame;
pub use manifest::{AdmArcadiumSection, AdmGameSection, AdmManifest};
pub use metadata::GameMetadata;

pub const SDK_VERSION: u32 = 1;
