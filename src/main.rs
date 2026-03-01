use std::{env::var, sync::Arc};

use axum::{
    Router,
    extract::{Path, State},
    response::Html,
    routing::get,
    serve,
};
use chrono::{NaiveDate, TimeDelta, Utc};
use dotenv::dotenv;
use gray_matter::{Matter, engine::TOML};
use markdown::{
    CompileOptions, Constructs, Options, ParseOptions, message::Message, to_html_with_options,
};
use serde::{Deserialize, Serialize};
use tera::Context;
use tokio::{
    fs::{File, read_dir},
    io::AsyncReadExt,
    net::TcpListener,
};
use tower_http::services::ServeDir;

mod error;
use error::{AppError, BuildError};
mod graphql;
mod state;
use state::{SharedState, Song};

const VERSION: &str = "4.2.0";

#[derive(Serialize, Deserialize)]
struct ContentInfo {
    title: String,
    date: Option<NaiveDate>,
    slug: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ProjectInfo {
    name: String,
    display_name: String,
    description: String,
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
            parse: ParseOptions {
                constructs: Constructs {
                    frontmatter: true,
                    gfm_table: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        },
    )
}

fn render_template(
    state: Arc<SharedState>,
    name: &str,
    context: &mut Context,
) -> Result<String, tera::Error> {
    context.insert("version", VERSION);
    context.insert("asset_url", &state.asset_url);
    state.tera.render(name, context)
}

fn format_time_delta(delta: &TimeDelta) -> String {
    let mut formatted = String::new();
    let (amount, unit) = if delta.num_weeks() > 0 {
        (delta.num_weeks(), "week")
    } else if delta.num_days() > 0 {
        (delta.num_days(), "day")
    } else if delta.num_hours() > 0 {
        (delta.num_hours(), "hour")
    } else if delta.num_minutes() > 0 {
        (delta.num_minutes(), "minute")
    } else {
        (delta.num_seconds(), "second")
    };
    formatted.push_str(&amount.to_string());
    formatted.push(' ');
    formatted.push_str(unit);
    if amount != 1 {
        formatted.push('s');
    }
    formatted.push_str(" ago");
    formatted
}

async fn index(State(state): State<Arc<SharedState>>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    add_music_to_context(state.clone(), &mut context).await;
    add_books_to_context(state.clone(), &mut context).await;
    add_projects_to_context(&mut context).await?;
    add_posts_to_context(&mut context).await?;
    Ok(Html(render_template(state, "index.html", &mut context)?))
}

async fn add_music_to_context(state: Arc<SharedState>, context: &mut Context) {
    if let Ok(track) = state.get_song().await {
        context.insert("music_success", &true);
        if let Some(track) = track {
            match track {
                Song::Now(current_track) => {
                    context.insert("track", &current_track);
                    context.insert("track_time", "now");
                }
                Song::Previous(previous_track) => {
                    let now = Utc::now();
                    context.insert("track", &previous_track);
                    context.insert(
                        "track_time",
                        &format_time_delta(&(now - previous_track.date).abs()),
                    );
                }
            }
        };
    }
}

async fn add_books_to_context(state: Arc<SharedState>, context: &mut Context) {
    if let Err(e) = state.get_books_and_goals().await {
        println!("{:?}", e);
    }
    if let Ok(data) = state.get_books_and_goals().await {
        context.insert("book_success", &true);
        if let Some(books_and_goals) = data {
            context.insert("books_and_goals", &books_and_goals);
        }
    }
}

async fn add_projects_to_context(context: &mut Context) -> Result<(), AppError> {
    let mut content = String::new();
    File::open("content/projects.json")
        .await?
        .read_to_string(&mut content)
        .await?;
    let projects = serde_json::from_str::<Vec<ProjectInfo>>(&content)?;
    context.insert("projects", &projects);
    Ok(())
}

async fn add_posts_to_context(context: &mut Context) -> Result<(), AppError> {
    let mut folder = read_dir("content/blog").await?;
    let mut posts = Vec::new();
    while let Some(entry) = folder.next_entry().await? {
        let mut content = String::new();
        if entry.file_type().await?.is_file() {
            let filename_lossy = entry.file_name().to_string_lossy().to_string();
            File::open(entry.path())
                .await?
                .read_to_string(&mut content)
                .await?;
            let frontmatter = Matter::<TOML>::new()
                .parse_with_struct::<ContentInfo>(&content)
                .ok_or(AppError::Frontmatter(filename_lossy))?
                .data;
            posts.push(frontmatter);
        }
    }
    posts.reverse();
    context.insert("posts", &posts);
    Ok(())
}

async fn fallback(State(state): State<Arc<SharedState>>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    Ok(Html(render_template(state, "404.html", &mut context)?))
}

async fn blog_post(
    State(state): State<Arc<SharedState>>,
    Path(slug): Path<String>,
) -> Result<Html<String>, AppError> {
    render_markdown(state, format!("content/blog/{}.md", slug), None).await
}

async fn changelog(State(state): State<Arc<SharedState>>) -> Result<Html<String>, AppError> {
    render_markdown(state, "content/changelog.md", Some("changelog")).await
}

async fn render_markdown<S: AsRef<str> + ToString>(
    state: Arc<SharedState>,
    file_path: S,
    custom_stylesheet: Option<&str>,
) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    let mut content = String::new();
    File::open(file_path.as_ref())
        .await?
        .read_to_string(&mut content)
        .await?;
    let content = content.replace("{{ asset_url }}", &state.asset_url);
    let frontmatter = Matter::<TOML>::new()
        .parse_with_struct::<ContentInfo>(&content)
        .ok_or(AppError::Frontmatter(file_path.to_string()))?
        .data;
    context.insert("title", &frontmatter.title);
    context.insert("content", &parse_markdown(&content)?);
    context.insert("date", &frontmatter.date);
    context.insert("custom_stylesheet", &custom_stylesheet);
    Ok(Html(render_template(state, "markdown.html", &mut context)?))
}

#[tokio::main]
async fn main() -> Result<(), BuildError> {
    if let Ok(dev_env) = var("DEVELOPMENT")
        && dev_env == "TRUE"
    {
        dotenv()?;
    }
    let state = SharedState::new("templates/**/*.html", "bobertoyin").await?;
    let app = Router::new()
        .route("/", get(index))
        .route("/blog/:slug", get(blog_post))
        .route("/changelog", get(changelog))
        .fallback(get(fallback))
        .with_state(Arc::new(state))
        .nest_service("/static", ServeDir::new("static"));
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    Ok(serve(listener, app).await?)
}
