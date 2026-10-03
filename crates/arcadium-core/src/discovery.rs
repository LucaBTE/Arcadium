use std::{
    error::Error,
    fs::{self, File},
    io::{self, Read},
    path::{Component, Path},
};

use arcadium_sdk::{AdmManifest, SDK_VERSION};

use zip::ZipArchive;

use crate::{
    installed_game::InstalledGame,
    limits::{MAX_ADM_BYTES, MAX_MANIFEST_BYTES, MAX_WASM_BYTES},
    registry::GameRegistry,
};

const MANIFEST_FILE: &str = "manifest.toml";

pub fn discover_games_from_directory(
    directory: &Path,
    registry: &mut GameRegistry,
) -> io::Result<()> {
    if !directory.exists() {
        return Ok(());
    }

    let mut games = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };

        if !extension.eq_ignore_ascii_case("adm") {
            continue;
        }

        match load_adm_package(&path) {
            Ok(game) => {
                games.push(game);
            }

            Err(error) => {
                eprintln!("Failed to load ADM package '{}': {}", path.display(), error);
            }
        }
    }

    games.sort_by(|left, right| {
        game_order(&left.metadata.id)
            .cmp(&game_order(&right.metadata.id))
            .then_with(|| left.source_path.cmp(&right.source_path))
    });
    for game in games {
        registry.register(game);
    }

    Ok(())
}

fn game_order(id: &str) -> u8 {
    match id {
        "com.arcadium.snake" => 0,
        "com.arcadium.pong" => 1,
        "com.arcadium.tictactoe" => 2,
        _ => 3,
    }
}

pub fn discover_games(bundled_directory: &Path, user_directory: &Path) -> io::Result<GameRegistry> {
    let mut registry = GameRegistry::new();

    discover_games_from_directory(bundled_directory, &mut registry)?;

    discover_games_from_directory(user_directory, &mut registry)?;

    Ok(registry)
}

pub(crate) fn load_adm_package(path: &Path) -> Result<InstalledGame, Box<dyn Error>> {
    let file = File::open(path)?;
    if file.metadata()?.len() > MAX_ADM_BYTES {
        return Err(invalid_data("ADM package exceeds the 16 MiB limit.").into());
    }

    let mut archive = ZipArchive::new(file)?;

    let manifest_content = read_manifest(&mut archive)?;

    let manifest: AdmManifest = toml::from_str(&manifest_content)?;

    validate_manifest(&manifest)?;

    validate_entry_size(&mut archive, &manifest.arcadium.entry)?;

    let metadata = manifest.metadata();

    Ok(InstalledGame::new(
        metadata,
        path.to_path_buf(),
        manifest.arcadium.sdk,
        manifest.arcadium.entry,
    ))
}

fn read_manifest(archive: &mut ZipArchive<File>) -> Result<String, Box<dyn Error>> {
    let mut manifest_file = archive
        .by_name(MANIFEST_FILE)
        .map_err(|_| invalid_data("ADM package is missing manifest.toml"))?;
    if manifest_file.size() > MAX_MANIFEST_BYTES {
        return Err(invalid_data("Manifest exceeds the 64 KiB limit.").into());
    }

    let mut content = String::new();

    manifest_file.read_to_string(&mut content)?;

    Ok(content)
}

fn validate_manifest(manifest: &AdmManifest) -> Result<(), Box<dyn Error>> {
    validate_required_field("game.id", &manifest.game.id)?;

    validate_required_field("game.name", &manifest.game.name)?;

    validate_required_field("game.author", &manifest.game.author)?;

    validate_required_field("game.version", &manifest.game.version)?;

    validate_required_field("game.description", &manifest.game.description)?;

    validate_required_field("arcadium.entry", &manifest.arcadium.entry)?;

    if manifest.arcadium.sdk != SDK_VERSION {
        return Err(invalid_data(format!(
            "unsupported Arcadium SDK version: package requires {}, \
                     but this Arcadium build supports {}",
            manifest.arcadium.sdk, SDK_VERSION
        ))
        .into());
    }

    validate_entry_path(&manifest.arcadium.entry)?;

    Ok(())
}

fn validate_required_field(field_name: &str, value: &str) -> Result<(), Box<dyn Error>> {
    if value.trim().is_empty() {
        return Err(
            invalid_data(format!("manifest field '{}' cannot be empty", field_name)).into(),
        );
    }

    Ok(())
}

fn validate_entry_path(entry: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(entry);

    if path.is_absolute() {
        return Err(invalid_data("arcadium.entry must be a relative path").into());
    }

    for component in path.components() {
        match component {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(invalid_data("arcadium.entry contains an invalid path").into());
            }

            _ => {}
        }
    }

    if !entry.ends_with(".wasm") {
        return Err(invalid_data("arcadium.entry must point to a .wasm file").into());
    }

    Ok(())
}

fn validate_entry_size(archive: &mut ZipArchive<File>, entry: &str) -> Result<(), Box<dyn Error>> {
    let wasm_file = archive.by_name(entry).map_err(|_| {
        invalid_data(format!(
            "ADM package declares entry '{}', \
                     but the file does not exist",
            entry
        ))
    })?;
    if wasm_file.size() > MAX_WASM_BYTES {
        return Err(invalid_data("WASM entry exceeds the 8 MiB limit.").into());
    }

    Ok(())
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}
