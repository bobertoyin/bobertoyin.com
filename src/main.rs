use std::{error::Error, sync::Arc};

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    serve, Router,
};
use markdown::{message::Message, to_html_with_options, CompileOptions, Options};
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
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            match self {
                AppError::Template(e) => e.to_string(),
                AppError::Io(e) => e.to_string(),
                AppError::Markdown(e) => e.to_string(),
            },
        )
            .into_response()
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

impl From<Message> for AppError {
    fn from(value: Message) -> Self {
        Self::Markdown(value)
    }
}

fn parse_markdown(content: &str) -> Result<String, Message> {
    // annoying that we have to allocate the Options every time
    // but currently Options is not Send/Sync: https://github.com/wooorm/markdown-rs/issues/104
    to_html_with_options(
        content,
        &Options {
            compile: CompileOptions {
                allow_dangerous_html: true,
                ..Default::default()
            },
            ..Default::default()
        },
    )
}

async fn index(State(tera): State<Arc<Tera>>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    let mut content = String::new();
    File::open("content/index.md")
        .await?
        .read_to_string(&mut content)
        .await?;
    context.insert("current_url", "/");

    context.insert("content", &parse_markdown(&content)?);
    Ok(Html(tera.render("index.html", &context)?))
}

async fn blog(State(tera): State<Arc<Tera>>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    context.insert("current_url", "/blog");
    Ok(Html(tera.render("blog.html", &context)?))
}

async fn projects(State(tera): State<Arc<Tera>>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    context.insert("current_url", "/projects");
    Ok(Html(tera.render("projects.html", &context)?))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let tera = Tera::new("templates/**/*.html")?;
    let app = Router::new()
        .route("/", get(index))
        .route("/blog", get(blog))
        .route("/projects", get(projects))
        .with_state(Arc::new(tera))
        .nest_service("/static", ServeDir::new("static"));
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    Ok(serve(listener, app).await?)
}
