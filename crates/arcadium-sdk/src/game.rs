use crate::GameMetadata;

pub trait ArcadeGame {
    fn metadata(&self) -> &GameMetadata;
}
