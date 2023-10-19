mod error;

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
        Q: Clone + Into<String> + Serialize,
        V: Clone + Serialize,
        for<'a> R: Deserialize<'a>,
    {
        let body = GraphQLRequest { query, variables };
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
