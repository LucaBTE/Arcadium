use std::{fs, io, path::PathBuf};

use crate::paths;

pub(crate) struct ScoreStore {
    path: PathBuf,
}

impl ScoreStore {
    #[cfg(test)]
    pub(crate) fn at_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub(crate) fn for_game(id: &str) -> io::Result<Self> {
        // Encode the manifest ID so a game cannot use path separators in its save name.
        let name = id
            .bytes()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        Ok(Self {
            path: paths::scores_directory()?.join(name),
        })
    }

    pub(crate) fn load(&self) -> u32 {
        fs::read_to_string(&self.path)
            .ok()
            .and_then(|value| value.trim().parse().ok())
            .unwrap_or(0)
    }

    pub(crate) fn save(&self, score: u32) -> bool {
        if score <= self.load() {
            return true;
        }
        let Some(parent) = self.path.parent() else {
            return false;
        };
        if fs::create_dir_all(parent).is_err() {
            return false;
        }
        let temporary = self
            .path
            .with_extension(format!("{}.tmp", std::process::id()));
        if fs::write(&temporary, score.to_string()).is_err() {
            return false;
        }
        if fs::rename(&temporary, &self.path).is_err() {
            let _ = fs::remove_file(temporary);
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn score_path_uses_centralized_directory_and_encodes_id() {
        let store = ScoreStore::for_game("a/b").unwrap();
        assert_eq!(
            store.path,
            paths::scores_directory().unwrap().join("612f62")
        );
    }

    #[test]
    fn score_survives_reopening_and_invalid_data_defaults_to_zero() {
        let path = env::temp_dir().join(format!("arcadium-score-test-{}", std::process::id()));
        let store = ScoreStore { path: path.clone() };
        let _ = fs::remove_file(&path);
        assert_eq!(store.load(), 0);
        assert!(store.save(42));
        assert_eq!(ScoreStore { path: path.clone() }.load(), 42);
        assert!(store.save(7));
        assert_eq!(store.load(), 42);
        fs::write(&path, "invalid").unwrap();
        assert_eq!(store.load(), 0);
        fs::remove_file(path).unwrap();
    }
}
