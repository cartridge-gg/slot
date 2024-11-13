#![allow(clippy::enum_variant_names)]

pub type Long = u64;
pub type Time = String;

mod accounts;
mod create;
mod delete;
mod describe;
mod fork;
mod list;
mod logs;
mod update;

pub use accounts::*;
pub use create::*;
pub use delete::*;
pub use describe::*;
pub use fork::*;
pub use list::*;
pub use logs::*;
pub use update::*;

/// Get the TOML config from the service config.
impl create::create_deployment::CreateServiceInput {
    pub fn get_config_toml(&self) -> Result<Option<String>, crate::error::Error> {
        match &self.config {
            Some(create::create_deployment::CreateServiceConfigInput {
                katana,
                torii,
                saya,
            }) => match (katana, torii, saya) {
                (Some(katana), None, None) => {
                    if let Some(config) = &katana.config_file {
                        Ok(Some(crate::read::base64_decode_string(&config)?))
                    } else {
                        Ok(None)
                    }
                }
                (None, Some(torii), None) => {
                    if let Some(config) = &torii.config_file {
                        Ok(Some(crate::read::base64_decode_string(&config)?))
                    } else {
                        Ok(None)
                    }
                }
                (None, None, Some(_)) => Ok(None),
                _ => Ok(None),
            },
            None => Ok(None),
        }
    }
}

/// Get the TOML config from the service config.
impl update::update_deployment::UpdateServiceInput {
    pub fn get_config_toml(&self) -> Result<Option<String>, crate::error::Error> {
        match &self.config {
            Some(update::update_deployment::UpdateServiceConfigInput { katana, torii }) => {
                match (katana, torii) {
                    (Some(katana), None) => {
                        if let Some(config) = &katana.config_file {
                            Ok(Some(crate::read::base64_decode_string(&config)?))
                        } else {
                            Ok(None)
                        }
                    }
                    (None, Some(torii)) => {
                        if let Some(config) = &torii.config_file {
                            Ok(Some(crate::read::base64_decode_string(&config)?))
                        } else {
                            Ok(None)
                        }
                    }
                    _ => Ok(None),
                }
            }
            None => Ok(None),
        }
    }
}
