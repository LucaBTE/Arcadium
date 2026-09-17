use arcadium_sdk::ArcadeGame;

//registry can contain every "concrete" type if it implements ArcadeGame
//registry.register(NewBeautifulGame::new());
//registry.register(InsaneGame::new());
//registry.register(TicTacToes::new());
pub struct GameRegistry {
    games: Vec<Box<dyn ArcadeGame>>,
}

impl GameRegistry {
    pub fn new() -> Self {
        Self { games: Vec::new() }
    }

    pub fn register<G>(&mut self, game: G)
    where
        G: ArcadeGame + 'static,
    {
        self.games.push(Box::new(game));
    }

    pub fn get(&self, index: usize) -> Option<&dyn ArcadeGame> {
        self.games.get(index).map(Box::as_ref)
    }

    pub fn iter(&self) -> impl Iterator<Item = &dyn ArcadeGame> {
        self.games.iter().map(Box::as_ref)
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
