use arcadium_core::GameRegistry;
use arcadium_game_dummy::DummyGame;
use arcadium_game_dummy2::DummyGame2;

fn main() -> std::io::Result<()> {
    let mut registry = GameRegistry::new();

    registry.register(DummyGame::new());
    registry.register(DummyGame2::new());

    arcadium_core::run(registry)
}
