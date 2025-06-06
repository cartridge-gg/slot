use crate::{error::Error, utils};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

const EULA_FILE: &str = "eula_acceptance.json";
const EULA_CONTENT_FILE: &str = "EULA.md";

/// Structure to store EULA acceptance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EulaAcceptance {
    /// SHA256 hash of the accepted EULA content
    pub content_hash: String,
    /// Timestamp when the EULA was accepted
    pub accepted_at: DateTime<Utc>,
    /// Version of the EULA (can be used to track material changes)
    pub version: String,
}

impl EulaAcceptance {
    /// Create a new EULA acceptance record
    pub fn new(content_hash: String, version: String) -> Self {
        Self {
            content_hash,
            accepted_at: Utc::now(),
            version,
        }
    }

    /// Store the EULA acceptance record
    pub fn store(&self) -> Result<PathBuf, Error> {
        Self::store_at(utils::config_dir(), self)
    }

    /// Load the stored EULA acceptance record
    pub fn load() -> Result<Option<Self>, Error> {
        Self::load_at(utils::config_dir())
    }

    fn store_at<P: AsRef<Path>>(config_dir: P, acceptance: &Self) -> Result<PathBuf, Error> {
        let path = get_file_path(config_dir);
        fs::create_dir_all(path.parent().expect("parent exists"))?;
        let content = serde_json::to_string_pretty(acceptance)?;
        fs::write(&path, content)?;
        Ok(path)
    }

    fn load_at<P: AsRef<Path>>(config_dir: P) -> Result<Option<Self>, Error> {
        let path = get_file_path(config_dir);

        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)?;
        match serde_json::from_str::<Self>(&content) {
            Ok(acceptance) => Ok(Some(acceptance)),
            Err(_) => {
                // If parsing fails, remove the malformed file
                let _ = fs::remove_file(&path);
                Ok(None)
            }
        }
    }
}

/// Get the path to the EULA acceptance file
fn get_file_path<P: AsRef<Path>>(config_dir: P) -> PathBuf {
    config_dir.as_ref().join(EULA_FILE)
}

/// Get the current EULA content
pub fn get_eula_content() -> Result<String, Error> {
    // First try to read from the package directory
    let eula_path = PathBuf::from(EULA_CONTENT_FILE);

    if eula_path.exists() {
        Ok(fs::read_to_string(eula_path)?)
    } else {
        // Fallback to embedded content
        Ok(include_str!("../../EULA.md").to_string())
    }
}

/// Calculate SHA256 hash of the EULA content
pub fn calculate_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Get the current EULA version
pub fn get_eula_version() -> &'static str {
    "1.0.0"
}

/// Check if the user has accepted the current EULA
pub fn has_accepted_current_eula() -> Result<bool, Error> {
    let acceptance = EulaAcceptance::load()?;

    match acceptance {
        Some(acc) => {
            let current_content = get_eula_content()?;
            let current_hash = calculate_content_hash(&current_content);
            let current_version = get_eula_version();

            // Check if the hash matches and version is current
            Ok(acc.content_hash == current_hash && acc.version == current_version)
        }
        None => Ok(false),
    }
}

/// Display EULA and prompt for acceptance
pub fn display_and_accept_eula() -> Result<bool, Error> {
    let content = get_eula_content()?;
    let lines: Vec<&str> = content.lines().collect();

    println!("\n{}\n", "=".repeat(80));
    println!("END USER LICENSE AGREEMENT (EULA)");
    println!("{}\n", "=".repeat(80));

    let mut current_line = 0;
    let lines_per_page = 20;

    loop {
        // Display current page
        for i in current_line..std::cmp::min(current_line + lines_per_page, lines.len()) {
            println!("{}", lines[i]);
        }

        current_line += lines_per_page;

        if current_line >= lines.len() {
            break;
        }

        println!("\n--- Press SPACE to continue, 'n' for next page, or 'q' to quit ---");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().to_lowercase().as_str() {
            "q" => return Ok(false),
            "n" | " " | "" => continue,
            _ => continue,
        }
    }

    // Display privacy policy URL
    println!("\n{}", "=".repeat(80));
    println!("Privacy Policy: https://cartridge.gg/privacy");
    println!("{}\n", "=".repeat(80));

    // Prompt for acceptance
    println!("\nBy typing 'I Accept', you agree to the terms of this End User License Agreement");
    println!("and acknowledge that you have read and understood our Privacy Policy.\n");
    print!("Type 'I Accept' to continue or anything else to cancel: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim() == "I Accept" {
        // Store acceptance
        let hash = calculate_content_hash(&content);
        let version = get_eula_version().to_string();
        let acceptance = EulaAcceptance::new(hash, version);
        acceptance.store()?;

        println!("\nThank you for accepting the EULA. You may now use the Slot CLI.\n");
        Ok(true)
    } else {
        println!("\nEULA not accepted. You must accept the EULA to use the Slot CLI.\n");
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils;

    #[test]
    fn test_eula_acceptance_rt() {
        let config_dir = utils::config_dir();
        let hash = "test_hash".to_string();
        let version = "1.0.0".to_string();

        let acceptance = EulaAcceptance::new(hash.clone(), version.clone());
        let _ = EulaAcceptance::store_at(&config_dir, &acceptance).unwrap();

        let loaded = EulaAcceptance::load_at(config_dir).unwrap();
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.content_hash, hash);
        assert_eq!(loaded.version, version);
    }

    #[test]
    fn test_content_hash() {
        let content = "This is a test EULA content";
        let hash1 = calculate_content_hash(content);
        let hash2 = calculate_content_hash(content);

        assert_eq!(hash1, hash2);

        let different_content = "This is different content";
        let hash3 = calculate_content_hash(different_content);

        assert_ne!(hash1, hash3);
    }
}
