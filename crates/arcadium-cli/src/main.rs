use std::{env, io, path::PathBuf};

use arcadium_core::discover_games;

fn main() -> io::Result<()> {
    let bundled_directory = bundled_games_directory();

    let user_directory = user_games_directory();

    ensure_user_games_directory(&user_directory)?;

    let registry = discover_games(&bundled_directory, &user_directory)?;

    arcadium_core::run(registry, bundled_directory, user_directory)
}

fn bundled_games_directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("bundled-games")
}

fn user_games_directory() -> PathBuf {
    if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data_home).join("arcadium").join("games");
    }

    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("arcadium")
            .join("games");
    }

    PathBuf::from(".").join("arcadium-data").join("games")
}

fn ensure_user_games_directory(path: &PathBuf) -> io::Result<()> {
    std::fs::create_dir_all(path)
}
