use regex::Regex;
use std::{fs, path::PathBuf, sync::OnceLock};

/// The default directory name where the Slot-generated files (e.g credentials/session keys) are stored.
const SLOT_DIR: &str = "slot";

/// Static instance of the email validation regex, compiled once on first use.
static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();

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

/// Validates if the provided string is a valid email address format.
///
/// Uses a regex pattern to check for basic email format:
/// - Local part: alphanumeric characters, dots, hyphens, underscores
/// - @ symbol
/// - Domain part: alphanumeric characters, dots, hyphens
/// - At least one dot in domain part
///
/// # Arguments
/// * `email` - The email string to validate
///
/// # Returns
/// * `true` if the email format is valid, `false` otherwise
pub fn is_valid_email(email: &str) -> bool {
    let regex = EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9._%+-]*[a-zA-Z0-9])?@[a-zA-Z0-9]([a-zA-Z0-9.-]*[a-zA-Z0-9])?\.[a-zA-Z]{2,}$").unwrap()
    });
    regex.is_match(email)
        && !email.contains("..")
        && !email.starts_with('.')
        && !email.ends_with('.')
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

    #[test]
    fn test_valid_emails() {
        assert!(super::is_valid_email("test@example.com"));
        assert!(super::is_valid_email("user.name@domain.co.uk"));
        assert!(super::is_valid_email("firstname+lastname@example.org"));
        assert!(super::is_valid_email("test_email@sub.domain.com"));
    }

    #[test]
    fn test_invalid_emails() {
        assert!(!super::is_valid_email("invalid-email"));
        assert!(!super::is_valid_email("@example.com"));
        assert!(!super::is_valid_email("test@"));
        assert!(!super::is_valid_email("test@.com"));
        assert!(!super::is_valid_email("test@domain"));
        assert!(!super::is_valid_email(""));
        assert!(!super::is_valid_email("test..email@example.com"));
    }

    #[test]
    fn test_edge_case_emails() {
        // Single character local/domain parts
        assert!(super::is_valid_email("a@b.com"));
        assert!(super::is_valid_email("x@example.co"));
        
        // Valid special characters
        assert!(super::is_valid_email("test-email@example.com"));
        assert!(super::is_valid_email("user_name@sub-domain.com"));
        
        // Domain with numbers
        assert!(super::is_valid_email("test@123domain.com"));
    }
}
