use std::{env, fs, io, path::PathBuf};

use arcadium_core::{discover_games, paths};

fn main() -> io::Result<()> {
    let bundled_directory = bundled_games_directory()?;
    let user_directory = paths::user_games_directory()?;
    fs::create_dir_all(&user_directory)?;

    let registry = discover_games(&bundled_directory, &user_directory)?;

    arcadium_core::run(registry, bundled_directory, user_directory)
}

fn bundled_games_directory() -> io::Result<PathBuf> {
    match env::var_os("ARCADIUM_BUNDLED_GAMES_DIR") {
        Some(path) => {
            let path = PathBuf::from(path);
            if !path.is_absolute() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "ARCADIUM_BUNDLED_GAMES_DIR must be an absolute path",
                ));
            }
            Ok(path)
        }
        None => paths::bundled_games_directory(),
    }
}
