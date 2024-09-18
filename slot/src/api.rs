use std::fmt::{self};

use graphql_client::Response;
use reqwest::RequestBuilder;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use url::Url;

use crate::error::Error;
use crate::{credential::AccessToken, vars};

#[derive(Debug)]
pub struct Client {
    base_url: Url,
    client: reqwest::Client,
    access_token: Option<AccessToken>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            access_token: None,
            client: reqwest::Client::new(),
            base_url: Url::parse(vars::get_cartridge_api_url().as_str()).expect("valid url"),
        }
    }

    pub fn new_with_token(token: AccessToken) -> Self {
        let mut client = Self::new();
        client.set_token(token);
        client
    }

    pub fn set_token(&mut self, token: AccessToken) {
        self.access_token = Some(token);
    }

    pub async fn query<R, T>(&self, body: &T) -> Result<R, Error>
    where
        R: DeserializeOwned,
        T: Serialize + ?Sized,
    {
        let path = "/query";
        let token = self.access_token.as_ref().map(|t| t.token.as_str());

        // TODO: return this as an error if token is None
        let bearer = format!("Bearer {}", token.unwrap_or_default());

        let response = self
            .post(path)
            .header("Authorization", bearer)
            .json(body)
            .send()
            .await?;

        if response.status() == 403 {
            return Err(Error::InvalidOAuth);
        }

        let res: Response<R> = response.json().await?;

        if let Some(errors) = res.errors {
            Err(Error::Api(GraphQLErrors(errors)))
        } else {
            Ok(res.data.unwrap())
        }
    }

    pub async fn oauth2(&self, code: &str) -> Result<AccessToken, Error> {
        #[derive(Deserialize)]
        struct OauthToken {
            #[serde(rename(deserialize = "access_token"))]
            token: String,
            #[serde(rename(deserialize = "token_type"))]
            r#type: String,
        }

        let path = "/oauth2/token";
        let form = [("code", code)];

        let response = self.post(path).form(&form).send().await?;
        let token: OauthToken = response.json().await?;

        Ok(AccessToken {
            token: token.token,
            r#type: token.r#type,
        })
    }

    fn post(&self, path: &str) -> RequestBuilder {
        let url = self.get_url(path);
        self.client.post(url)
    }

    fn get_url(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();
        url.path_segments_mut().unwrap().extend(path.split('/'));
        url
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub struct GraphQLErrors(Vec<graphql_client::Error>);

impl fmt::Display for GraphQLErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for err in &self.0 {
            writeln!(f, "Error: {}\n", err.message)?;
        }
        Ok(())
    }
}
