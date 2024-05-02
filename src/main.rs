use std::{error::Error, sync::Arc};

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    serve, Router,
};
use markdown::{to_html_with_options, CompileOptions, Options, ParseOptions};
use tera::{Context, Tera};
use tokio::{fs::File, io::AsyncReadExt, net::TcpListener};
use tower_http::services::ServeDir;

enum AppError {
    Template(tera::Error),
    Io(std::io::Error),
    Markdown(markdown::message::Message),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, match self {
            AppError::Template(e) => e.to_string(),
            AppError::Io(e) => e.to_string(),
            AppError::Markdown(e) => e.to_string(),
        }).into_response()
        
    }
}

impl From<tera::Error> for AppError {
    fn from(value: tera::Error) -> Self {
        Self::Template(value)
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<markdown::message::Message> for AppError {
    fn from(value: markdown::message::Message) -> Self {
        Self::Markdown(value)
    }
}
 
async fn index(State(tera): State<Arc<Tera>>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    let mut content = String::new();
    File::open("content/index.md").await?.read_to_string(&mut content).await?;
    context.insert("current_url", "/");

    context.insert("content", &to_html_with_options(&content, &Options { parse: ParseOptions::default(), compile: CompileOptions { allow_dangerous_html: true, ..Default::default()} })?);
    Ok(Html(tera.render("index.html", &context)?))
}

async fn blog(State(tera): State<Arc<Tera>>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    context.insert("current_url", "/blog");
    Ok(Html(tera.render("blog.html", &context)?))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let tera = Tera::new("templates/**/*.html")?;
    let app = Router::new()
        .route("/", get(index))
        .route("/blog", get(blog))
        .with_state(Arc::new(tera))
        .nest_service("/static", ServeDir::new("static"));
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    Ok(serve(listener, app).await?)
}
