use crate::installed_game::InstalledGame;

pub struct GameRegistry {
    games: Vec<InstalledGame>,
}

impl GameRegistry {
    pub fn new() -> Self {
        Self { games: Vec::new() }
    }

    pub fn register(&mut self, game: InstalledGame) {
        if !self
            .games
            .iter()
            .any(|existing| existing.metadata.id == game.metadata.id)
        {
            self.games.push(game);
        }
    }

    pub fn get(&self, index: usize) -> Option<&InstalledGame> {
        self.games.get(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &InstalledGame> {
        self.games.iter()
    }

    pub fn len(&self) -> usize {
        self.games.len()
    }

    pub fn is_empty(&self) -> bool {
        self.games.is_empty()
    }
}

impl Default for GameRegistry {
    fn default() -> Self {
        Self::new()
    }
}
