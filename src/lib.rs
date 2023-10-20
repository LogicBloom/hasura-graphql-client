mod error;

use reqwest::Client;
use secrecy::ExposeSecret;
use secrecy::Secret;
use serde::{Deserialize, Serialize};

use error::HasuraError;
pub use error::HasuraGraphQLClientError;

#[derive(Clone, Debug)]
pub struct HasuraGraphQLClient {
    api_url: String,
    admin_secret: Secret<String>,
    http_client: Client,
}

impl HasuraGraphQLClient {
    pub fn new(api_url: &str, admin_secret: &str) -> Self {
        let http_client = Client::default();
        Self {
            http_client,
            admin_secret: Secret::new(admin_secret.into()),
            api_url: api_url.into(),
        }
    }

    pub async fn post_query<R, V>(
        &self,
        query: &str,
        variables: Option<V>,
        bearer_token: Option<&str>,
    ) -> Result<R, HasuraGraphQLClientError>
    where
        for<'a> R: Deserialize<'a>,
        V: Clone + Serialize,
    {
        let body = GraphQLRequest { query, variables };
        let mut builder = self.http_client.post(&self.api_url).json(&body);
        if let Some(token) = bearer_token {
            builder = builder.header("Authorization", format!("Bearer {}", token))
        } else {
            builder = builder.header("x-hasura-admin-secret", self.admin_secret.expose_secret());
        }
        let result = builder
            .send()
            .await?
            .error_for_status()?
            .json::<serde_json::Value>()
            .await?;
        if let Some(errors) = result.get("errors") {
            let errors = serde_json::from_value::<Vec<HasuraError>>(errors.to_owned())?;
            return Err(HasuraGraphQLClientError::GraphqlError(errors));
        }
        let result = result.get("data").ok_or(anyhow::anyhow!(
            "Invalid response body: missing the 'data' property"
        ))?;
        Ok(serde_json::from_value::<R>(result.to_owned())?)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct GraphQLRequest<Q: Into<String>, V> {
    query: Q,
    variables: Option<V>,
}
