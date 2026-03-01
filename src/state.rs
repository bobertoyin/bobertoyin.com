use std::env::var;

use chrono::{Datelike, Utc};
use futures_util::{StreamExt, pin_mut};
use lastfm::{
    Client,
    errors::Error as LastFMError,
    track::{NowPlayingTrack, RecordedTrack},
};
use moka::future::Cache;
use tera::Tera;
use tokio::time::Duration;

use crate::error::{AppError, BuildError};
use crate::graphql::{Client as GQLClient, Data, Me, Vars};

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

        let hardcover = GQLClient::build(HARDCOVER_API_URL, hardcover_auth_token)?;
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

    pub async fn get_books_and_goals(&self) -> Result<Option<Me>, Vec<AppError>> {
        match self.hardcover_cache.get(&()).await {
            Some(me) => Ok(Some(me)),
            None => {
                let date = format!("{}-12-31", Utc::now().year());
                match self
                    .hardcover
                    .query::<Data>("graphql/hardcover/query.graphql", Vars { date })
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
