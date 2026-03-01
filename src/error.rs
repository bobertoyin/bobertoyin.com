use std::env::VarError;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use gql_client::GraphQLError;
use markdown::message::Message;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Template engine error: {0}")]
    Template(#[from] tera::Error),
    #[error("Error for environment variable \"{0}\": {1}")]
    EnvVar(&'static str, VarError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error(".env error: {0}")]
    DotEnv(#[from] dotenv::Error),
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Template engine error: {0}")]
    Template(#[from] tera::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Markdown error: {0}")]
    Markdown(Message),
    #[error("Markdown frontmatter error for file: {0}")]
    Frontmatter(String),
    #[error("Json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("GraphQL error: {0}")]
    GraphQLError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

impl From<Message> for AppError {
    fn from(value: Message) -> Self {
        Self::Markdown(value)
    }
}

impl From<GraphQLError> for AppError {
    fn from(value: GraphQLError) -> Self {
        Self::GraphQLError(value.to_string())
    }
}
