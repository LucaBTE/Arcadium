use std::{fs, io, path::Path};

use arcadium_sdk::AdmManifest;

use crate::{installed_game::InstalledGame, registry::GameRegistry};

pub fn discover_games_from_directory(
    directory: &Path,
    registry: &mut GameRegistry,
) -> io::Result<()> {
    if !directory.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };

        if extension != "adm" {
            continue;
        }

        match load_adm_manifest(&path) {
            Ok(game) => {
                registry.register(game);
            }

            Err(error) => {
                eprintln!("Failed to load ADM file '{}': {}", path.display(), error);
            }
        }
    }

    Ok(())
}

pub fn discover_games(bundled_directory: &Path, user_directory: &Path) -> io::Result<GameRegistry> {
    let mut registry = GameRegistry::new();

    discover_games_from_directory(bundled_directory, &mut registry)?;

    discover_games_from_directory(user_directory, &mut registry)?;

    Ok(registry)
}

fn load_adm_manifest(path: &Path) -> Result<InstalledGame, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;

    let manifest: AdmManifest = toml::from_str(&content)?;

    let metadata = manifest.metadata();

    Ok(InstalledGame::new(
        metadata,
        path.to_path_buf(),
        manifest.arcadium.sdk,
    ))
}
