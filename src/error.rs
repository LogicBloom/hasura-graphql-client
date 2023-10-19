use serde::Deserialize;

#[derive(thiserror::Error, Debug)]
pub enum HasuraGraphQLClientError {
    #[error("GraphQL request failed: {0:?}")]
    GraphqlError(Vec<HasuraError>),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    InternalError(#[from] anyhow::Error),
    #[error("Serde error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
pub struct HasuraError {
    pub message: String,
    pub extensions: HasuraErrorExtension,
}

#[derive(Debug, Deserialize)]
pub struct HasuraErrorExtension {
    pub code: String,
    pub path: String,
}
