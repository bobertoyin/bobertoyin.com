use std::{fmt::Display, path::Path};

use reqwest::{
    Client as ReqClient,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::{fs::File, io::AsyncReadExt};

use crate::error::{AppError, BuildError, GraphQLError};

#[derive(Clone, Deserialize)]
pub struct Data {
    pub me: Vec<Me>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Me {
    pub goals: Vec<Goal>,
    pub user_books: Vec<UserBook>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Goal {
    pub id: i32,
    pub description: String,
    pub metric: String,
    pub progress: f64,
    pub goal: i32,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct UserBook {
    pub book: Book,
    pub user_book_reads: Vec<UserBookRead>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Book {
    pub title: String,
    pub slug: String,
    pub image: Image,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Image {
    pub url: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct UserBookRead {
    pub progress: f64,
}

#[derive(Serialize)]
pub struct Vars {
    pub date: String,
}

#[derive(Serialize)]
pub struct RequestBody<T: Serialize> {
    pub query: String,
    pub variables: T,
}

#[derive(Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Clone)]
pub struct Client {
    client: ReqClient,
    url: String,
}

impl Client {
    pub fn build(url: impl Into<String>, auth_token: impl Display) -> Result<Self, BuildError> {
        let mut auth_token_header = HeaderValue::from_str(&format!("Bearer {}", auth_token))?;
        auth_token_header.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.append(AUTHORIZATION, auth_token_header);
        let client = ReqClient::builder()
            .use_native_tls()
            .default_headers(headers)
            .build()?;
        Ok(Self {
            client,
            url: url.into(),
        })
    }

    pub async fn query<D: DeserializeOwned>(
        &self,
        query_file_path: impl AsRef<Path>,
        variables: impl Serialize,
    ) -> Result<Option<D>, Vec<AppError>> {
        let mut query = String::new();
        File::open(query_file_path)
            .await
            .map_err(AppError::from)?
            .read_to_string(&mut query)
            .await
            .map_err(AppError::from)?;

        let body = RequestBody { query, variables };
        let response = self
            .client
            .post(self.url.as_str())
            .json(&body)
            .send()
            .await
            .map_err(AppError::from)?;

        let json = response
            .json::<GraphQLResponse<D>>()
            .await
            .map_err(AppError::from)?;

        if let Some(errors) = json.errors {
            return Err(errors.into_iter().map(AppError::from).collect());
        }

        Ok(json.data)
    }
}
