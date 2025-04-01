use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{env, fs};

use crate::account::AccountInfo;
use crate::error::Error;
use crate::utils::{self};

const CREDENTIALS_FILE: &str = "credentials.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccessToken {
    pub token: String,
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Credentials {
    pub account: AccountInfo,
    pub access_token: AccessToken,
}

impl Credentials {
    pub fn new(account: AccountInfo, access_token: AccessToken) -> Self {
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
        let content = if let Ok(slot_auth) = env::var("SLOT_AUTH") {
            slot_auth
        } else {
            let path = get_file_path(config_dir);

            if !path.exists() {
                return Err(Error::Unauthorized);
            }

            fs::read_to_string(path)?
        };

        serde_json::from_str::<Credentials>(&content).map_err(|_| Error::MalformedCredentials)
    }
}

/// Get the path to the credentials file.
pub fn get_file_path<P: AsRef<Path>>(config_dir: P) -> PathBuf {
    config_dir.as_ref().join(CREDENTIALS_FILE)
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use serde_json::{json, Value};

    use crate::account::AccountInfo;
    use crate::credential::{AccessToken, Credentials, CREDENTIALS_FILE};
    use crate::{utils, Error};
    use std::fs;

    // This test is to make sure that changes made to the `Credentials` struct doesn't
    // introduce breaking changes to the serde format.
    #[test]
    fn test_rt_static_format() {
        let json = json!({
            "account": {
                "id": "foo",
                "username": "username",
                "controllers": [
                    {
                        "id": "foo",
                        "address": "0x12345",
                        "signers": [
                            {
                                "id": "bar",
                                "type": "WebAuthn"
                            }
                        ]
                    }
                ],
                "credentials": [
                    {
                        "id": "foobar",
                        "publicKey": "mypublickey"
                    }
                ]
            },
            "access_token": {
                "token": "oauthtoken",
                "type": "bearer"
            }
        });

        let credentials: Credentials = serde_json::from_value(json.clone()).unwrap();

        assert_eq!(credentials.account.id, "foo".to_string());
        assert_eq!(credentials.account.username, "username".to_string());
        assert_eq!(credentials.account.credentials[0].id, "foobar");
        assert_eq!(credentials.account.credentials[0].public_key, "mypublickey");
        assert_eq!(credentials.access_token.token, "oauthtoken");
        assert_eq!(credentials.access_token.r#type, "bearer");

        let credentials_serialized: Value = serde_json::to_value(&credentials).unwrap();
        assert_eq!(json, credentials_serialized);
    }

    #[test]
    fn loading_malformed_credentials() {
        let malformed_cred = json!({
            "access_token": "mytoken",
            "token_type": "mytokentype"
        });

        let dir = utils::config_dir();
        let path = dir.join(CREDENTIALS_FILE);
        fs::create_dir_all(&dir).expect("failed to create intermediary dirs");
        fs::write(path, serde_json::to_vec(&malformed_cred).unwrap()).unwrap();

        let result = Credentials::load_at(dir);
        assert_matches!(result, Err(Error::MalformedCredentials))
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

        let expected = Credentials::new(AccountInfo::default(), access_token);
        let _ = Credentials::store_at(&config_dir, &expected).unwrap();

        let actual = Credentials::load_at(config_dir).unwrap();
        assert_eq!(expected, actual);
    }
}
