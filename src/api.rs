use graphql_client::Response;
use serde::{de::DeserializeOwned, Serialize};

use crate::credential::Credentials;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    ReqwestError(reqwest::Error),
    #[error(transparent)]
    CredentialsError(anyhow::Error),
}

pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            base_url: "https://api.cartridge.gg/query".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn post<R: DeserializeOwned, T: Serialize + ?Sized>(
        &self,
        body: &T,
    ) -> Result<Response<R>, ApiError> {
        let credentials = Credentials::load()
            .map_err(|_| {
                anyhow::anyhow!("Failed to load credentials. Login with `slot auth login`.")
            })
            .map_err(ApiError::CredentialsError)?;

        let res = self
            .client
            .post(&self.base_url)
            .header(
                "Authorization",
                format!("Bearer {}", credentials.access_token),
            )
            .json(body)
            .send()
            .await
            .map_err(ApiError::ReqwestError)?;
        let res: Response<R> = res.json().await.map_err(ApiError::ReqwestError)?;

        Ok(res)
    }
}
