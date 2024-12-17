use std::collections::HashSet;

use aws_config::{environment::EnvironmentVariableCredentialsProvider, from_env, Region};
use aws_sdk_s3::{Client as S3Client, Error as S3Error};
use futures_util::{pin_mut, StreamExt};
use lastfm::{
    track::{NowPlayingTrack, RecordedTrack},
    Client,
};
use moka::future::Cache;
use octocrab::{
    models::{repos::Languages, Repository},
    Octocrab, OctocrabBuilder,
};
use serde::{Deserialize, Serialize};
use tera::Tera;
use tokio::time::Duration;

use crate::error::{AppError, BuildError};

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

#[derive(Clone, Serialize, Deserialize)]
pub struct PhotoFolder {
    pub name: String,
    pub display_name: String,
}

impl From<String> for PhotoFolder {
    fn from(value: String) -> Self {
        Self {
            name: value.clone(),
            display_name: value.replace("-", " "),
        }
    }
}

/// Various structs and state that are shared across routes.
/// Mostly for storing API clients and the like.
#[derive(Clone)]
pub struct SharedState {
    pub tera: Tera,
    pub lastfm: Client<String, String>,
    pub lastfm_cache: Cache<(), Option<Song>>,
    pub github: Octocrab,
    pub github_cache: Cache<String, (Repository, Languages)>,
    github_username: String,
    pub s3: S3Client,
    s3_bucket: String,
}

impl SharedState {
    pub async fn new(
        template_path: &str,
        lastfm_username: &str,
        github_username: &str,
        s3_endpoint: &str,
        s3_region: Region,
        s3_bucket: &str,
    ) -> Result<Self, BuildError> {
        let tera = Tera::new(template_path)?;
        let lastfm = Client::<String, String>::try_from_env(lastfm_username.to_string())
            .map_err(|err| BuildError::EnvVar("LASTFM_API_KEY", err))?;
        let lastfm_cache = Cache::<(), Option<Song>>::builder()
            .time_to_live(Duration::from_secs(2))
            .build();
        let github = OctocrabBuilder::default().build()?;
        let github_cache = Cache::<String, (Repository, Languages)>::builder()
            .time_to_live(Duration::from_secs(3 * 60))
            .build();
        let s3 = S3Client::new(
            &from_env()
                .endpoint_url(s3_endpoint)
                .region(s3_region)
                .credentials_provider(EnvironmentVariableCredentialsProvider::new())
                .load()
                .await,
        );
        Ok(Self {
            tera,
            lastfm,
            lastfm_cache,
            github,
            github_cache,
            github_username: github_username.to_string(),
            s3,
            s3_bucket: s3_bucket.to_string(),
        })
    }

    pub async fn get_repo(&self, name: &str) -> Result<(Repository, Languages), AppError> {
        match self.github_cache.get(name).await {
            Some(info) => Ok(info),
            None => {
                let repo = self.github.repos(&self.github_username, name).get().await?;
                let languages = self
                    .github
                    .repos(&self.github_username, name)
                    .list_languages()
                    .await?;
                self.github_cache
                    .insert(name.to_string(), (repo.clone(), languages.clone()))
                    .await;
                Ok((repo, languages))
            }
        }
    }

    pub async fn get_song(&self) -> Result<Option<Song>, AppError> {
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

    pub async fn get_photo_directories(&self) -> Result<Vec<PhotoFolder>, S3Error> {
        Ok(self
            .s3
            .list_objects_v2()
            .bucket(&self.s3_bucket)
            .prefix("")
            .delimiter("/")
            .send()
            .await
            .map_err(Into::<S3Error>::into)?
            .common_prefixes()
            .iter()
            .map(|x| PhotoFolder::from(x.prefix().unwrap_or_default().replace("/", "")))
            .collect())
    }

    pub async fn existing_directory(&self, dir: &str) -> Result<bool, S3Error> {
        let dirs = self
            .get_photo_directories()
            .await?
            .into_iter()
            .map(|f| f.name)
            .collect::<HashSet<String>>();
        Ok(dirs.contains(dir))
    }

    pub async fn get_photos(&self, folder: &str) -> Result<Vec<String>, S3Error> {
        let mut folder = folder.to_string();
        if !folder.ends_with("/") {
            folder += "/";
        }
        Ok(self
            .s3
            .list_objects_v2()
            .bucket(&self.s3_bucket)
            .prefix(&folder)
            .delimiter("/")
            .send()
            .await
            .map_err(Into::<S3Error>::into)?
            .contents()
            .iter()
            .map(|x| x.key().unwrap_or_default().to_string())
            .filter(|x| !x.ends_with('/'))
            .collect())
    }
}
