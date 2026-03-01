use std::{collections::HashMap, env::var};

use chrono::{Datelike, Utc};
use futures_util::{StreamExt, pin_mut};
use gql_client::Client as GQLClient;
use lastfm::{
    Client,
    errors::Error as LastFMError,
    track::{NowPlayingTrack, RecordedTrack},
};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use tera::Tera;
use tokio::{fs::File, io::AsyncReadExt, time::Duration};

use crate::error::{AppError, BuildError};

const HARDCOVER_API_URL: &str = "https://api.hardcover.app/v1/graphql";

#[derive(Clone)]
pub enum Song {
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

#[derive(Clone, Deserialize)]
struct Data {
    me: Vec<Me>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Me {
    goals: Vec<Goal>,
    user_books: Vec<UserBook>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Goal {
    id: i32,
    description: String,
    metric: String,
    progress: f64,
    goal: i32,
}

#[derive(Clone, Deserialize, Serialize)]
struct UserBook {
    book: Book,
    user_book_reads: Vec<UserBookRead>,
}

#[derive(Clone, Deserialize, Serialize)]
struct Book {
    title: String,
    slug: String,
    image: Image,
}

#[derive(Clone, Deserialize, Serialize)]
struct Image {
    url: String,
}

#[derive(Clone, Deserialize, Serialize)]
struct UserBookRead {
    progress: f64,
}

#[derive(Serialize)]
struct Vars {
    date: String,
}

/// Various structs and state that are shared across routes.
/// Mostly for storing API clients and the like.
#[derive(Clone)]
pub struct SharedState {
    pub tera: Tera,
    pub lastfm: Client<String, String>,
    pub lastfm_cache: Cache<(), Option<Song>>,
    pub hardcover: GQLClient,
    pub hardcover_cache: Cache<(), Me>,
}

impl SharedState {
    pub async fn new(template_path: &str, lastfm_username: &str) -> Result<Self, BuildError> {
        let tera = Tera::new(template_path)?;

        let lastfm = Client::<String, String>::try_from_env(lastfm_username.to_string())
            .map_err(|err| BuildError::EnvVar("LASTFM_API_KEY", err))?;
        let lastfm_cache = Cache::<(), Option<Song>>::builder()
            .time_to_live(Duration::from_secs(15))
            .build();

        let hardcover_auth_token = var("HARDCOVER_AUTH_TOKEN")
            .map_err(|err| BuildError::EnvVar("HARDCOVER_AUTH_TOKEN", err))?;
        let hardcover_auth_token = format!("Bearer {}", hardcover_auth_token);
        let mut hardcover_headers = HashMap::new();
        hardcover_headers.insert("authorization", hardcover_auth_token);

        let hardcover = GQLClient::new_with_headers(HARDCOVER_API_URL, hardcover_headers);
        let hardcover_cache = Cache::<(), Me>::builder()
            .time_to_live(Duration::from_mins(15))
            .build();

        Ok(Self {
            tera,
            lastfm,
            lastfm_cache,
            hardcover,
            hardcover_cache,
        })
    }

    pub async fn get_song(&self) -> Result<Option<Song>, LastFMError> {
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

    pub async fn get_books_and_goals(&self) -> Result<Option<Me>, AppError> {
        match self.hardcover_cache.get(&()).await {
            Some(me) => Ok(Some(me)),
            None => {
                let mut query = String::new();
                File::open("graphql/hardcover/query.graphql")
                    .await?
                    .read_to_string(&mut query)
                    .await?;
                let date = format!("{}-12-31", Utc::now().year());
                match self
                    .hardcover
                    .query_with_vars::<Data, Vars>(&query, Vars { date })
                    .await?
                {
                    None => Ok(None),
                    Some(data) => {
                        let me = data.me[0].clone();
                        self.hardcover_cache.insert((), me.clone()).await;
                        Ok(Some(me))
                    }
                }
            }
        }
    }
}
