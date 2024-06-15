use std::path::PathBuf;

/// The default directory name where the Slot-generated files (e.g credentials/session keys) are stored.
pub const SLOT_DIR: &str = "slot";

/// Get the path to the config directory where the Slot-generated files (e.g credentials/session keys) are stored.
/// If this function is called in a test environment, path to a temporary directory is returned instead.
pub fn config_dir() -> PathBuf {
    if cfg!(test) {
        tempfile::tempdir().unwrap().into_path()
    } else {
        dirs::config_local_dir().expect("unsupported OS")
    }
}
