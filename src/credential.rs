use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self};
use std::path::{Path, PathBuf};

use crate::graphql::auth::me::MeMe;

const SLOT_DIR: &str = "slot";
const CREDENTIALS_FILE: &str = "credentials.json";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error("No credentials found, please authenticate with `slot auth login`")]
    Unauthorized,
    #[error("Legacy credentials found, please reauthenticate with `slot auth login`")]
    LegacyCredentials,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub token: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyCredentials {
    access_token: String,
    token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    #[serde(flatten)]
    pub account: Option<MeMe>,
    pub access_token: AccessToken,
}

impl Credentials {
    pub fn new(account: Option<MeMe>, access_token: AccessToken) -> Self {
        Self {
            account,
            access_token,
        }
    }

    pub fn load() -> Result<Self, Error> {
        load_at_path(get_file_path())
    }

    pub fn write(&self) -> io::Result<()> {
        // create the dir paths if it doesn't yet exist
        let path = get_file_path();
        fs::create_dir_all(path.parent().expect("qed; parent exist"))?;

        let content = serde_json::to_string_pretty(&self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

fn load_at_path<P: AsRef<Path>>(path: P) -> Result<Credentials, Error> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(Error::Unauthorized);
    }

    let content = fs::read_to_string(path)?;
    let credentials = serde_json::from_str::<Credentials>(&content);

    match credentials {
        Ok(creds) => Ok(creds),
        Err(_) => {
            // check if the file is in the legacy format
            let legacy = serde_json::from_str::<LegacyCredentials>(&content);
            match legacy {
                Ok(_) => Err(Error::LegacyCredentials),
                Err(e) => Err(Error::IO(e.into())),
            }
        }
    }
}

/// Get the path to the credentials file.
fn get_file_path() -> PathBuf {
    let mut path = dirs::config_local_dir().unwrap();
    path.extend([SLOT_DIR, CREDENTIALS_FILE]);
    path
}

#[cfg(test)]
mod tests {
    use super::{load_at_path, LegacyCredentials};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn loading_legacy_credentials() {
        let cred = LegacyCredentials {
            access_token: "mytoken".to_string(),
            token_type: "mytokentype".to_string(),
        };

        let dir = tempdir().unwrap();
        let path = dir.path().join("cred.json");
        fs::write(&path, serde_json::to_vec(&cred).unwrap()).unwrap();

        let err = load_at_path(path).unwrap_err();
        assert!(err.to_string().contains("Legacy credentials found"))
    }
}
