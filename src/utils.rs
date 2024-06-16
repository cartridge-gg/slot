use std::{fs, path::PathBuf};

/// The default directory name where the Slot-generated files (e.g credentials/session keys) are stored.
const SLOT_DIR: &str = "slot";

/// Get the path to the config directory where the Slot-generated files (e.g credentials/session keys) are stored.
///  This function guarantees that the config directory exists.
///
/// If this function is called in a test environment, path to a temporary directory is returned instead.
pub fn config_dir() -> PathBuf {
    let path = if cfg!(test) {
        tempfile::tempdir().unwrap().into_path()
    } else {
        dirs::config_local_dir().expect("unsupported OS")
    }
    .join(SLOT_DIR);

    if path.exists() {
        path
    } else {
        fs::create_dir_all(&path).expect("failed to create config directory");
        path
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::SLOT_DIR;

    #[test]
    fn config_dir_must_exist() {
        let path = super::config_dir();
        assert!(path.exists());
        assert!(path.ends_with(SLOT_DIR));
    }
}
