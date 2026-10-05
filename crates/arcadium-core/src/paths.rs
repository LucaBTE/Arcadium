use std::{env, ffi::OsStr, io, path::PathBuf};

fn absolute_path(value: &OsStr, name: &str) -> io::Result<PathBuf> {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        Ok(path)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name} must be an absolute path"),
        ))
    }
}

fn home_directory() -> io::Result<PathBuf> {
    let home = env::var_os("HOME").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "HOME is not set; cannot locate Arcadium data",
        )
    })?;
    absolute_path(&home, "HOME")
}

#[cfg(target_os = "linux")]
pub fn data_directory() -> io::Result<PathBuf> {
    linux_data_directory(env::var_os("XDG_DATA_HOME").as_deref())
}

#[cfg(target_os = "linux")]
fn linux_data_directory(xdg_data_home: Option<&OsStr>) -> io::Result<PathBuf> {
    match xdg_data_home {
        Some(path) => Ok(absolute_path(path, "XDG_DATA_HOME")?.join("arcadium")),
        None => Ok(home_directory()?.join(".local/share/arcadium")),
    }
}

#[cfg(target_os = "macos")]
pub fn data_directory() -> io::Result<PathBuf> {
    Ok(home_directory()?.join("Library/Application Support/Arcadium"))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn data_directory() -> io::Result<PathBuf> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Arcadium supports Linux and macOS only",
    ))
}

pub fn bundled_games_directory() -> io::Result<PathBuf> {
    Ok(data_directory()?.join("bundled-games"))
}

pub fn user_games_directory() -> io::Result<PathBuf> {
    Ok(data_directory()?.join("games"))
}

pub fn scores_directory() -> io::Result<PathBuf> {
    Ok(data_directory()?.join("scores"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_paths_are_required() {
        assert!(absolute_path(OsStr::new("/tmp/data"), "DATA").is_ok());
        assert!(absolute_path(OsStr::new("relative/data"), "DATA").is_err());
        assert!(absolute_path(OsStr::new(""), "DATA").is_err());
    }

    #[test]
    fn subdirectories_share_the_data_root() {
        let root = data_directory().unwrap();
        assert_eq!(
            bundled_games_directory().unwrap(),
            root.join("bundled-games")
        );
        assert_eq!(user_games_directory().unwrap(), root.join("games"));
        assert_eq!(scores_directory().unwrap(), root.join("scores"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_xdg_data_home_takes_precedence_and_must_be_absolute() {
        assert_eq!(
            linux_data_directory(Some(OsStr::new("/tmp/xdg-data"))).unwrap(),
            PathBuf::from("/tmp/xdg-data/arcadium")
        );
        assert!(linux_data_directory(Some(OsStr::new("relative"))).is_err());
    }
}
