use std::{env::var, sync::Arc};

use aws_config::Region;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    serve, Router,
};
use chrono::{NaiveDate, TimeDelta, Utc};
use dotenv::dotenv;
use futures_util::future::try_join_all;
use gray_matter::{engine::TOML, Matter};
use markdown::{
    message::Message, to_html_with_options, CompileOptions, Constructs, Options, ParseOptions,
};
use octocrab::models::{repos::Languages, Repository};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use tokio::{
    fs::{read_dir, File},
    io::AsyncReadExt,
    net::TcpListener,
    spawn,
};
use tower_http::services::ServeDir;

mod error;
use error::{AppError, BuildError};
mod state;
use state::{SharedState, Song};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize)]
struct BlogInfo {
    title: String,
    date: NaiveDate,
    slug: String,
}

#[derive(Serialize, Deserialize)]
struct PageInfo {
    title: String,
}

#[derive(Serialize, Deserialize)]
struct ProjectInfo {
    name: String,
    display_name: String,
    description: String,
    url: Option<String>,
    #[serde(default)]
    in_progress: bool,
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

fn render_template(tera: &Tera, name: &str, context: &mut Context) -> Result<String, tera::Error> {
    context.insert("version", VERSION);
    tera.render(name, context)
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

fn title_case<S: AsRef<str>>(string: S) -> String {
    let words = string
        .as_ref()
        .split(" ")
        .map(|word| {
            let mut word = word.chars().collect::<Vec<char>>();
            word[0] = word[0].to_ascii_uppercase();
            word.into_iter().collect()
        })
        .collect::<Vec<String>>();
    words.join(" ")
}

async fn index(State(state): State<Arc<SharedState>>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    let mut content = String::new();
    File::open("content/index.md")
        .await?
        .read_to_string(&mut content)
        .await?;
    let frontmatter = Matter::<TOML>::new()
        .parse_with_struct::<PageInfo>(&content)
        .ok_or(AppError::Frontmatter("content/index.md".to_string()))?
        .data;
    context.insert("title", &frontmatter.title);
    context.insert("active", &frontmatter.title.to_lowercase());
    context.insert("content", &parse_markdown(&content)?);
    Ok(Html(render_template(
        &state.tera,
        "basic.html",
        &mut context,
    )?))
}

async fn blog(State(state): State<Arc<SharedState>>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
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
                .parse_with_struct::<BlogInfo>(&content)
                .ok_or(AppError::Frontmatter(filename_lossy))?
                .data;
            posts.push(frontmatter);
        }
    }
    posts.reverse();
    context.insert("active", "blog");
    context.insert("posts", &posts);
    Ok(Html(render_template(
        &state.tera,
        "blog.html",
        &mut context,
    )?))
}

async fn blog_post(
    State(state): State<Arc<SharedState>>,
    Path(slug): Path<String>,
) -> Result<Html<String>, AppError> {
    let file_path = format!("content/blog/{}.md", slug);
    let mut context = Context::new();
    let mut content = String::new();
    File::open(&file_path)
        .await?
        .read_to_string(&mut content)
        .await?;
    let frontmatter = Matter::<TOML>::new()
        .parse_with_struct::<BlogInfo>(&content)
        .ok_or(AppError::Frontmatter(file_path))?
        .data;
    context.insert("post", &frontmatter);
    context.insert("content", &parse_markdown(&content)?);
    Ok(Html(render_template(
        &state.tera,
        "blog_post.html",
        &mut context,
    )?))
}

async fn projects(State(state): State<Arc<SharedState>>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    let mut content = String::new();
    File::open("content/projects.json")
        .await?
        .read_to_string(&mut content)
        .await?;
    let projects = serde_json::from_str::<Vec<ProjectInfo>>(&content)?;
    let mut repo_data_handles = Vec::new();
    for project in projects.iter() {
        let state_clone = state.clone();
        let name_clone = project.name.clone();
        repo_data_handles.push(spawn(
            async move { state_clone.get_repo(&name_clone).await },
        ));
    }
    let repo_data: Result<Vec<(Repository, Languages)>, AppError> =
        try_join_all(repo_data_handles).await?.into_iter().collect();
    context.insert("projects", &projects);
    context.insert("repo_data", &repo_data?);
    context.insert("active", "projects");
    Ok(Html(render_template(
        &state.tera,
        "projects.html",
        &mut context,
    )?))
}

async fn changelog(State(state): State<Arc<SharedState>>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    let mut content = String::new();
    File::open("content/changelog.md")
        .await?
        .read_to_string(&mut content)
        .await?;
    let frontmatter = Matter::<TOML>::new()
        .parse_with_struct::<PageInfo>(&content)
        .ok_or(AppError::Frontmatter("content/changelog.md".to_string()))?
        .data;
    context.insert("title", &frontmatter.title);
    context.insert("content", &parse_markdown(&content)?);
    Ok(Html(render_template(
        &state.tera,
        "basic.html",
        &mut context,
    )?))
}

async fn currently_playing(
    State(state): State<Arc<SharedState>>,
) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    let track = state.get_song().await?;
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
    match render_template(&state.tera, "currently_playing.html", &mut context) {
        Ok(content) => Ok(Html(content)),
        Err(e) => Ok(Html(format!(
            "<span id=\"track\" class=\"has-text-danger\">{}</span>",
            e
        ))),
    }
}

async fn photos_home(State(state): State<Arc<SharedState>>) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    context.insert("active", "photos");
    context.insert("folders", &state.get_photo_directories().await?);
    Ok(Html(render_template(
        &state.tera,
        "photos_home.html",
        &mut context,
    )?))
}

async fn photos_folder(
    State(state): State<Arc<SharedState>>,
    Path(folder): Path<String>,
) -> Result<Response, AppError> {
    let mut context = Context::new();
    context.insert("active", "photos");
    if !state.existing_directory(&folder).await? {
        return Ok(StatusCode::NOT_FOUND.into_response());
    }
    context.insert("images", &state.get_photos(&folder).await?);
    context.insert("photo_folder", &title_case(folder.replace("-", " ")));
    Ok(Html(render_template(
        &state.tera,
        "photos_folder.html",
        &mut context,
    )?)
    .into_response())
}

#[tokio::main]
async fn main() -> Result<(), BuildError> {
    if let Ok(dev_env) = var("DEVELOPMENT") {
        if dev_env == "TRUE" {
            dotenv()?;
        }
    }
    let state = SharedState::new(
        "templates/**/*.html",
        "bobertoyin",
        "bobertoyin",
        "https://nyc3.digitaloceanspaces.com",
        Region::new("nyc3"),
        "bobertoyin-photos",
    )
    .await?;
    let app = Router::new()
        .route("/", get(index))
        .route("/blog", get(blog))
        .route("/blog/:slug", get(blog_post))
        .route("/projects", get(projects))
        .route("/changelog", get(changelog))
        .route("/currently_playing", get(currently_playing))
        .route("/photos", get(photos_home))
        .route("/photos/:folder", get(photos_folder))
        .with_state(Arc::new(state))
        .nest_service("/static", ServeDir::new("static"));
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    Ok(serve(listener, app).await?)
}
