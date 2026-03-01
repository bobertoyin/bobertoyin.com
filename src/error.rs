use std::{
    collections::HashMap,
    env::VarError,
    fmt::{Display, Formatter, Result as FmtResult},
};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use markdown::message::Message;
use reqwest::{Error as ReqError, header::InvalidHeaderValue};
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug, Clone, Error)]
pub struct GraphQLError {
    pub message: String,
    pub locations: Option<Vec<GraphQLErrorLocation>>,
    pub extensions: Option<HashMap<String, String>>,
    pub path: Option<Vec<GraphQLErrorPathParam>>,
}

impl Display for GraphQLError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message)?;
        if let Some(locations) = &self.locations {
            write!(f, ", {:#?}", locations)?;
        }
        if let Some(extensions) = &self.extensions {
            write!(f, ", {:#?}", extensions)?;
        }
        if let Some(path) = &self.path {
            write!(f, ", {:#?}", path)?;
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct GraphQLErrorLocation {
    pub line: u32,
    pub column: u32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum GraphQLErrorPathParam {
    String(String),
    Number(u32),
}

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
    #[error("HTTP header value error: {0}")]
    HTTPHeaderValue(#[from] InvalidHeaderValue),
    #[error("HTTP client error: {0}")]
    HTTPClient(#[from] ReqError),
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
    #[error("HTTP client error: {0}")]
    HTTPClient(#[from] ReqError),
    #[error["GraphQL errors: {0}"]]
    GraphQL(#[from] GraphQLError),
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

impl From<AppError> for Vec<AppError> {
    fn from(value: AppError) -> Self {
        vec![value]
    }
}
