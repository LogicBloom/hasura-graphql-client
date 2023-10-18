mod error;

use anyhow::Context;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use error::HasuraError;
pub use error::HasuraGraphQLClientError;

#[derive(Debug, Clone)]
pub struct HasuraGraphQLClient {
    http_client: Client,
    api_url: String,
    hasura_admin_secret: String,
}

impl HasuraGraphQLClient {
    pub fn new<T: Into<String>>(api_url: T, hasura_admin_secret: T) -> Self {
        let http_client = Client::default();
        Self {
            http_client,
            hasura_admin_secret: hasura_admin_secret.into(),
            api_url: api_url.into(),
        }
    }

    pub async fn post_query<Q, V, R>(
        &self,
        query: Q,
        variables: Option<V>,
        bearer_token: Option<Q>,
    ) -> Result<R, HasuraGraphQLClientError>
    where
        Q: Into<String> + Serialize,
        V: Serialize,
        for<'a> R: Deserialize<'a>,
    {
        let body = serde_json::json!({
            "query": query,
            "variables": variables
        });
        let mut builder = self.http_client.post(&self.api_url).json(&body);
        if let Some(token) = bearer_token {
            builder = builder.bearer_auth(token.into());
        } else {
            builder = builder.header("x-hasura-admin-secret", &self.hasura_admin_secret);
        }
        let result = builder
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await
            .context("Failed to deserialize response body to JSON")?;
        if result.get("errors").is_some() {
            // safe to unwrap since we know the value is some
            let errors = serde_json::from_value::<Vec<HasuraError>>(
                result.get("errors").unwrap().to_owned(),
            )
            .context("Failed to deserialize to Vec<HasuraError>")?;
            return Err(HasuraGraphQLClientError::GraphqlError(errors));
        }
        let result = result
            .get("data")
            .ok_or_else(|| anyhow::anyhow!("Invalid response body: missing the 'data' property"))?;
        Ok(serde_json::from_value::<R>(result.to_owned()).map_err(|e| anyhow::anyhow!("{e:?}"))?)
    }
}
