use actix_web::{
    HttpResponse, post,
    web::{self, Data},
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Debug, Serialize, Deserialize)]
struct UserInput {
    // user: RegisterUser,
    #[serde(rename = "text_title")]
    title: String,
    #[serde(rename = "text_body")]
    body: String,
    #[serde(rename = "text_footer")]
    footer: String,
    #[serde(default)]
    date: chrono::DateTime<chrono::Utc>,
}

impl Default for UserInput {
    fn default() -> Self {
        Self {
            title: String::new(),
            body: String::new(),
            footer: String::new(),
            date: DateTime::now(),
        }
    }
}

#[instrument(
    name = "User submits text",
    level = "info",
    target = "Personal journal",
    skip(body, _mongo, _redis)
)]
#[post("/submit_text")]
pub async fn submit_text(
    _mongo: Data<mongodb::Client>,
    _redis: Data<r2d2::Pool<redis::Client>>,
    body: web::Form<UserInput>,
) -> HttpResponse {
    tracing::info!("Submit text endpoint");

    tracing::warn!("body: {body:#?}");
    HttpResponse::Ok().body("Text received")
}
