use std::{env, fs, path::PathBuf};

pub(crate) struct ScoreStore {
    path: PathBuf,
}

impl ScoreStore {
    #[cfg(test)]
    pub(crate) fn at_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub(crate) fn for_game(id: &str) -> Self {
        let root = if let Ok(path) = env::var("XDG_DATA_HOME") {
            PathBuf::from(path)
        } else if let Ok(home) = env::var("HOME") {
            PathBuf::from(home).join(".local/share")
        } else {
            PathBuf::from("arcadium-data")
        };
        // Encode the manifest ID so a game cannot use path separators in its save name.
        let name = id
            .bytes()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        Self {
            path: root.join("arcadium/scores").join(name),
        }
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
