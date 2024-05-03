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

const VERSION: &str = env!("CARGO_PKG_VERSION");

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

fn render_template(tera: &Tera, name: &str, context: &mut Context) -> Result<String, tera::Error> {
    context.insert("version", VERSION);
    tera.render(name, context)
}

async fn index(State(tera): State<Arc<Tera>>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    let mut content = String::new();
    File::open("content/index.md")
        .await?
        .read_to_string(&mut content)
        .await?;
    context.insert("active", "home");

    context.insert("content", &parse_markdown(&content)?);
    context.insert("basic_title", "home");
    Ok(Html(render_template(&tera, "basic.html", &mut context)?))
}

async fn blog(State(tera): State<Arc<Tera>>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    context.insert("active", "blog");
    Ok(Html(render_template(&tera, "blog.html", &mut context)?))
}

async fn projects(State(tera): State<Arc<Tera>>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    context.insert("active", "projects");
    Ok(Html(render_template(&tera, "projects.html", &mut context)?))
}

async fn changelog(State(tera): State<Arc<Tera>>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    let mut content = String::new();
    File::open("content/changelog.md")
        .await?
        .read_to_string(&mut content)
        .await?;

    context.insert("content", &parse_markdown(&content)?);
    context.insert("basic_title", "changelog");
    Ok(Html(render_template(&tera, "basic.html", &mut context)?))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let tera = Tera::new("templates/**/*.html")?;
    let app = Router::new()
        .route("/", get(index))
        .route("/blog", get(blog))
        .route("/projects", get(projects))
        .route("/changelog", get(changelog))
        .with_state(Arc::new(tera))
        .nest_service("/static", ServeDir::new("static"));
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    Ok(serve(listener, app).await?)
}
