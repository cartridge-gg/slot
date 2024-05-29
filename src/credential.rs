use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self};
use std::path::PathBuf;

use crate::command::auth::info::me::MeMe;

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
pub struct LegacyCredentials {
    pub access_token: String,
    pub token_type: String,
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
        let path = get_file_path();
        if !path.exists() {
            return Err(Error::Unauthorized);
        }

        let content = fs::read_to_string(path.clone())?;
        let credentials: Result<Credentials, _> = serde_json::from_str(&content);
        match credentials {
            Ok(creds) => Ok(creds),
            Err(_) => Err(Error::LegacyCredentials),
        }
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

/// Get the path to the credentials file.
fn get_file_path() -> PathBuf {
    let mut path = dirs::config_local_dir().unwrap();
    path.extend([SLOT_DIR, CREDENTIALS_FILE]);
    path
}
