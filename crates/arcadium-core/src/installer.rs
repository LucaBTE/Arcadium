use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    discovery::{discover_games_from_directory, load_adm_package},
    registry::GameRegistry,
};

static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

pub(crate) struct InstallResult {
    pub id: String,
    pub name: String,
    pub updated: bool,
}

pub(crate) fn install_game(
    source: &Path,
    bundled_directory: &Path,
    user_directory: &Path,
) -> io::Result<InstallResult> {
    if !source.is_file() {
        return Err(invalid_input("Selected path is not a file."));
    }
    if !source
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("adm"))
    {
        return Err(invalid_input("Select an .adm file."));
    }

    let game = load_adm_package(source).map_err(|error| {
        eprintln!(
            "Could not validate ADM package '{}': {error}",
            source.display()
        );
        if error
            .to_string()
            .contains("unsupported Arcadium SDK version")
        {
            invalid_input("Unsupported SDK version.")
        } else {
            invalid_input("Invalid ADM package.")
        }
    })?;

    let mut bundled = GameRegistry::new();
    discover_games_from_directory(bundled_directory, &mut bundled)?;
    if bundled
        .iter()
        .any(|existing| existing.metadata.id == game.metadata.id)
    {
        return Err(invalid_input("A bundled game already uses this game ID."));
    }

    let mut user = GameRegistry::new();
    discover_games_from_directory(user_directory, &mut user)?;
    let existing = user
        .iter()
        .find(|existing| existing.metadata.id == game.metadata.id);
    let destination = if let Some(existing) = existing {
        existing.source_path.clone()
    } else {
        let filename = source
            .file_name()
            .ok_or_else(|| invalid_input("Select an .adm file."))?;
        let destination = user_directory.join(filename);
        if destination.exists() {
            return Err(invalid_input(
                "A different game already uses this filename.",
            ));
        }
        destination
    };

    fs::create_dir_all(user_directory)?;
    let temporary = loop {
        let number = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
        let path = user_directory.join(format!(
            ".arcadium-install-{}-{number}.tmp",
            std::process::id()
        ));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut output) => {
                let result = (|| {
                    let mut input = fs::File::open(source)?;
                    io::copy(&mut input, &mut output)?;
                    output.flush()?;
                    Ok::<(), io::Error>(())
                })();
                if let Err(error) = result {
                    let _ = fs::remove_file(&path);
                    return Err(error);
                }
                break path;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };
    if let Err(error) = fs::rename(&temporary, &destination) {
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }

    Ok(InstallResult {
        id: game.metadata.id,
        name: game.metadata.name,
        updated: existing.is_some(),
    })
}

fn invalid_input(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{app::App, discover_games};
    use std::{path::PathBuf, sync::atomic::AtomicU64};
    use zip::{ZipWriter, write::SimpleFileOptions};

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let number = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "arcadium-install-test-{}-{number}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn path(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }

        fn package(&self, name: &str, id: &str, title: &str) -> PathBuf {
            let path = self.path(name);
            let file = fs::File::create(&path).unwrap();
            let mut archive = ZipWriter::new(file);
            archive
                .start_file("manifest.toml", SimpleFileOptions::default())
                .unwrap();
            write!(archive, "[game]\nid = \"{id}\"\nname = \"{title}\"\nauthor = \"Tester\"\nversion = \"1\"\ndescription = \"Test\"\n[arcadium]\nsdk = 1\nentry = \"game.wasm\"\n").unwrap();
            archive
                .start_file("game.wasm", SimpleFileOptions::default())
                .unwrap();
            archive.write_all(b"wasm").unwrap();
            archive.finish().unwrap();
            path
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn valid_installation_is_discoverable() {
        let fixture = Fixture::new();
        let source = fixture.package("new.adm", "community.new", "New Game");
        let bundled = fixture.path("bundled");
        let user = fixture.path("user");
        let installed = install_game(&source, &bundled, &user).unwrap();
        assert!(!installed.updated);
        assert!(user.join("new.adm").is_file());
        assert_eq!(
            discover_games(&bundled, &user)
                .unwrap()
                .get(0)
                .unwrap()
                .metadata
                .id,
            installed.id
        );
    }

    #[test]
    fn invalid_package_leaves_directory_and_registry_intact() {
        let fixture = Fixture::new();
        let source = fixture.path("invalid.adm");
        fs::write(&source, b"not a zip").unwrap();
        let bundled = fixture.path("bundled");
        let user = fixture.path("user");
        assert!(install_game(&source, &bundled, &user).is_err());
        assert!(!user.exists());
        assert!(discover_games(&bundled, &user).unwrap().is_empty());
    }

    #[test]
    fn bundled_id_cannot_be_replaced() {
        let fixture = Fixture::new();
        let bundled = fixture.path("bundled");
        fs::create_dir(&bundled).unwrap();
        let original = fixture.package("original.adm", "same.id", "Bundled");
        let bundled_file = bundled.join("original.adm");
        fs::copy(&original, &bundled_file).unwrap();
        let source = fixture.package("replacement.adm", "same.id", "Replacement");
        let before = fs::read(&bundled_file).unwrap();
        assert!(install_game(&source, &bundled, &fixture.path("user")).is_err());
        assert_eq!(fs::read(&bundled_file).unwrap(), before);
    }

    #[test]
    fn update_replaces_existing_path_and_keeps_one_entry() {
        let fixture = Fixture::new();
        let bundled = fixture.path("bundled");
        let user = fixture.path("user");
        let first = fixture.package("first.adm", "same.id", "Old");
        install_game(&first, &bundled, &user).unwrap();
        let second = fixture.package("second.adm", "same.id", "New");
        assert!(install_game(&second, &bundled, &user).unwrap().updated);
        assert!(user.join("first.adm").exists());
        assert!(!user.join("second.adm").exists());
        let registry = discover_games(&bundled, &user).unwrap();
        assert_eq!(registry.len(), 1);
        assert_eq!(registry.get(0).unwrap().metadata.name, "New");
    }

    #[test]
    fn same_filename_cannot_overwrite_another_game() {
        let fixture = Fixture::new();
        let bundled = fixture.path("bundled");
        let user = fixture.path("user");
        let first = fixture.package("same.adm", "first.id", "First");
        install_game(&first, &bundled, &user).unwrap();
        let before = fs::read(user.join("same.adm")).unwrap();
        let other = fixture.path("other");
        fs::create_dir(&other).unwrap();
        let second = fixture.package("second.adm", "second.id", "Second");
        fs::copy(&second, other.join("same.adm")).unwrap();
        assert!(install_game(&other.join("same.adm"), &bundled, &user).is_err());
        assert_eq!(fs::read(user.join("same.adm")).unwrap(), before);
    }

    #[test]
    fn library_refresh_selects_installed_game_and_cancel_does_nothing() {
        let fixture = Fixture::new();
        let bundled = fixture.path("bundled");
        let user = fixture.path("user");
        fs::create_dir(&bundled).unwrap();
        let bundled_source = fixture.package("bundled-source.adm", "bundled.id", "Bundled");
        fs::copy(bundled_source, bundled.join("bundled.adm")).unwrap();
        let mut app = App::new(
            discover_games(&bundled, &user).unwrap(),
            bundled,
            user.clone(),
        );
        assert!(!app.take_install_request());
        assert!(!user.exists());
        assert_eq!(app.game_count(), 1);
        let source = fixture.package("new.adm", "community.new", "New Game");
        app.install_selected_file(&source);
        assert_eq!(app.game_count(), 2);
        assert_eq!(app.selected_game().unwrap().metadata.id, "community.new");
        assert_eq!(
            app.library_message.as_deref(),
            Some("Installed \"New Game\".")
        );
        assert!(app.selected_game < app.game_count());
    }

    #[test]
    fn discovery_keeps_bundled_precedence_and_deduplicates_user_ids() {
        let fixture = Fixture::new();
        let bundled = fixture.path("bundled");
        let user = fixture.path("user");
        fs::create_dir(&bundled).unwrap();
        fs::create_dir(&user).unwrap();
        let bundled_source = fixture.package("bundled-source.adm", "same.id", "Bundled");
        fs::copy(bundled_source, bundled.join("bundled.adm")).unwrap();
        let user_source = fixture.package("user-source.adm", "same.id", "User");
        fs::copy(user_source, user.join("a.adm")).unwrap();
        let other_source = fixture.package("other-source.adm", "other.id", "First");
        fs::copy(&other_source, user.join("c.adm")).unwrap();
        fs::copy(other_source, user.join("b.adm")).unwrap();
        let registry = discover_games(&bundled, &user).unwrap();
        assert_eq!(registry.len(), 2);
        assert_eq!(registry.get(0).unwrap().metadata.name, "Bundled");
        assert_eq!(registry.get(1).unwrap().source_path, user.join("b.adm"));
    }
}
