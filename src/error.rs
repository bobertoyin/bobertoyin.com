use std::env::VarError;

use aws_sdk_s3::Error as S3Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use markdown::message::Message;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Template engine error: {0}")]
    Template(#[from] tera::Error),
    #[error("Error for environment variable \"{0}\": {1}")]
    EnvVar(&'static str, VarError),
    #[error("Github error: {0}")]
    Github(#[from] octocrab::Error),
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
    #[error("Last.fm error: {0}")]
    LastFm(#[from] lastfm::errors::Error),
    #[error("Github error: {0}")]
    Github(#[from] octocrab::Error),
    #[error("Json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Task join error: {0}")]
    Task(#[from] JoinError),
    #[error("S3 error: {0}")]
    S3(#[from] S3Error),
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
