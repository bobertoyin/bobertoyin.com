use std::{env::VarError, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    serve, Router,
};
use chrono::{NaiveDate, TimeDelta, Utc};
use futures_util::{future::try_join_all, pin_mut, StreamExt};
use gray_matter::{engine::TOML, Matter};
use lastfm::{
    track::{NowPlayingTrack, RecordedTrack},
    Client,
};
use markdown::{
    message::Message, to_html_with_options, CompileOptions, Constructs, Options, ParseOptions,
};
use moka::future::Cache;
use octocrab::{
    models::{repos::Languages, Repository},
    Octocrab, OctocrabBuilder,
};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use thiserror::Error;
use tokio::{
    fs::{read_dir, File},
    io::AsyncReadExt,
    net::TcpListener,
    spawn,
    task::JoinError,
    time::Duration,
};
use tower_http::services::ServeDir;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone)]
struct SharedState {
    tera: Tera,
    lastfm: Client<String, String>,
    lastfm_cache: Cache<(), Option<Song>>,
    github: Octocrab,
    github_cache: Cache<String, (Repository, Languages)>,
}

impl SharedState {
    async fn get_repo(&self, name: &str) -> Result<(Repository, Languages), AppError> {
        match self.github_cache.get(name).await {
            Some(info) => Ok(info),
            None => {
                let repo = self.github.repos("bobertoyin", name).get().await?;
                let languages = self
                    .github
                    .repos("bobertoyin", name)
                    .list_languages()
                    .await?;
                self.github_cache
                    .insert(name.to_string(), (repo.clone(), languages.clone()))
                    .await;
                Ok((repo, languages))
            }
        }
    }

    async fn get_song(&self) -> Result<Option<Song>, AppError> {
        match self.lastfm_cache.get(&()).await {
            Some(song) => Ok(song),
            None => {
                let track = self.lastfm.now_playing().await?;
                if let Some(track) = track {
                    let wrapped = Song::from(track);
                    self.lastfm_cache.insert((), Some(wrapped.clone())).await;
                    Ok(Some(wrapped))
                } else {
                    let stream = self.lastfm.clone().all_tracks().await?.into_stream();
                    pin_mut!(stream);
                    match stream.next().await {
                        Some(track) => {
                            let wrapped = Song::from(track?);
                            self.lastfm_cache.insert((), Some(wrapped.clone())).await;
                            Ok(Some(wrapped))
                        }
                        None => {
                            self.lastfm_cache.insert((), None).await;
                            Ok(None)
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
enum Song {
    Now(NowPlayingTrack),
    Previous(RecordedTrack),
}

impl From<NowPlayingTrack> for Song {
    fn from(value: NowPlayingTrack) -> Self {
        Self::Now(value)
    }
}

impl From<RecordedTrack> for Song {
    fn from(value: RecordedTrack) -> Self {
        Self::Previous(value)
    }
}

#[derive(Debug, Error)]
enum BuildError {
    #[error("Template engine error: {0}")]
    Template(#[from] tera::Error),
    #[error("Error for environment variable \"{0}\": {1}")]
    EnvVar(&'static str, VarError),
    #[error("Github error: {0}")]
    Github(#[from] octocrab::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
enum AppError {
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
    #[serde(default)]
    missing_photo: bool,
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
        "blog-post.html",
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
    match render_template(&state.tera, "currently-playing.html", &mut context) {
        Ok(content) => Ok(Html(content)),
        Err(e) => Ok(Html(format!(
            "<span id=\"track\" class=\"has-text-danger\">{}</span>",
            e
        ))),
    }
}

#[tokio::main]
async fn main() -> Result<(), BuildError> {
    let tera = Tera::new("templates/**/*.html")?;
    let lastfm = Client::<String, String>::try_from_env("bobertoyin".to_string())
        .map_err(|err| BuildError::EnvVar("LASTFM_API_KEY", err))?;
    let lastfm_cache = Cache::<(), Option<Song>>::builder()
        .time_to_live(Duration::from_secs(2))
        .build();
    let github = OctocrabBuilder::default().build()?;
    let github_cache = Cache::<String, (Repository, Languages)>::builder()
        .time_to_live(Duration::from_secs(3 * 60))
        .build();
    let app = Router::new()
        .route("/", get(index))
        .route("/blog", get(blog))
        .route("/blog/:slug", get(blog_post))
        .route("/projects", get(projects))
        .route("/changelog", get(changelog))
        .route("/currently_playing", get(currently_playing))
        .with_state(Arc::new(SharedState {
            tera,
            lastfm,
            lastfm_cache,
            github,
            github_cache,
        }))
        .nest_service("/static", ServeDir::new("static"));
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    Ok(serve(listener, app).await?)
}
