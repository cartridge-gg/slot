use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self};
use std::path::{Path, PathBuf};

use crate::graphql::auth::me::MeMe;
use crate::utils::{self};

const CREDENTIALS_FILE: &str = "credentials.json";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error("No credentials found, please authenticate with `slot auth login`")]
    Unauthorized,

    #[error("Legacy credentials found, please reauthenticate with `slot auth login`")]
    LegacyCredentials,

    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccessToken {
    pub token: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyCredentials {
    access_token: String,
    token_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

    /// Load the credentials of the currently authenticated user.
    ///
    /// # Errors
    ///
    /// This function will fail if no user has authenticated yet, or if
    /// the credentials file are invalid or missing.
    ///
    pub fn load() -> Result<Self, Error> {
        Self::load_at(utils::config_dir())
    }

    /// Store the credentials of an authenticated user. Returns the path to the stored credentials
    /// file.
    pub fn store(&self) -> Result<PathBuf, Error> {
        Self::store_at(utils::config_dir(), self)
    }

    pub(crate) fn store_at<P: AsRef<Path>>(
        config_dir: P,
        credentials: &Self,
    ) -> Result<PathBuf, Error> {
        let path = get_file_path(config_dir);
        // create the dir paths if it doesn't yet exist
        fs::create_dir_all(path.parent().expect("qed; parent exist"))?;
        let content = serde_json::to_string_pretty(credentials)?;
        fs::write(&path, content)?;
        Ok(path)
    }

    pub(crate) fn load_at<P: AsRef<Path>>(config_dir: P) -> Result<Credentials, Error> {
        let path = get_file_path(config_dir);

        println!("{}", path.display());

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
}

/// Get the path to the credentials file.
pub fn get_file_path<P: AsRef<Path>>(config_dir: P) -> PathBuf {
    config_dir.as_ref().join(CREDENTIALS_FILE)
}

#[cfg(test)]
mod tests {
    use super::LegacyCredentials;
    use crate::credential::{AccessToken, Credentials, CREDENTIALS_FILE};
    use crate::utils;
    use std::fs;

    #[test]
    fn loading_legacy_credentials() {
        let cred = LegacyCredentials {
            access_token: "mytoken".to_string(),
            token_type: "mytokentype".to_string(),
        };

        let dir = utils::config_dir();
        let path = dir.join(CREDENTIALS_FILE);
        fs::create_dir_all(&dir).expect("failed to create intermediary dirs");
        fs::write(path, serde_json::to_vec(&cred).unwrap()).unwrap();

        let err = Credentials::load_at(dir).unwrap_err();
        assert!(err.to_string().contains("Legacy credentials found"))
    }

    #[test]
    fn loading_non_existent_credentials() {
        let dir = utils::config_dir();
        let err = Credentials::load_at(dir).unwrap_err();
        assert!(err.to_string().contains("No credentials found"))
    }

    #[test]
    fn credentials_rt() {
        let config_dir = utils::config_dir();

        let access_token = AccessToken {
            token: "mytoken".to_string(),
            r#type: "Bearer".to_string(),
        };

        let expected = Credentials::new(None, access_token);
        let _ = Credentials::store_at(&config_dir, &expected).unwrap();

        let actual = Credentials::load_at(config_dir).unwrap();
        assert_eq!(expected, actual);
    }
}
