use std::{error::Error, sync::Arc};

use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    serve, Router,
};
use tera::{Context, Tera};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

struct TemplateError(tera::Error);

impl IntoResponse for TemplateError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0.to_string()).into_response()
    }
}

impl From<tera::Error> for TemplateError {
    fn from(value: tera::Error) -> Self {
        Self(value)
    }
}

async fn index(State(tera): State<Arc<Tera>>) -> Result<Html<String>, TemplateError> {
    let mut context = Context::new();
    context.insert("current_url", "/");
    context.insert("job_searching", &false);
    context.insert("location", "Madison, WI");
    context.insert("work_title", "Software Developer");
    context.insert("work_location", "Epic Systems");
    context.insert("work_location_url", "https://epic.com");
    context.insert("education", "BSCS");
    context.insert("education_location", "Northeastern University");
    context.insert("education_location_url", "https://northeastern.edu");

    context.insert("email", "bobertoyin@gmail.com");
    context.insert("resume", "resume.pdf");
    context.insert("github", "bobertoyin");
    context.insert("linkedin", "boberto");
    Ok(Html(tera.render("index.html", &context)?))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let tera = Tera::new("templates/**/*.html")?;
    let app = Router::new()
        .route("/", get(index))
        .with_state(Arc::new(tera))
        .nest_service("/static", ServeDir::new("static"));
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    Ok(serve(listener, app).await?)
}
