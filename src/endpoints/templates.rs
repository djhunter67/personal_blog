use actix_files::NamedFile;
use actix_web::{HttpResponse, Responder, get};
use askama::Template;
use std::path::PathBuf;
use tracing::{error, info, instrument};

use super::user_input::BlogPost;

/// # TODO
///
/// Dark Theme
#[derive(Template, Default)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: String,

    pub content: Vec<BlogPost>,
    pub version: String,
    pub user_email: String,
    pub is_logged_in: bool,
}

impl IndexTemplate {
    /// Creates a new [`IndexTemplate`].
    #[must_use]
    pub fn new(
        title: String,
        content: Vec<BlogPost>,
        user_email: String,
        is_logged_in: bool,
    ) -> Self {
        Self {
            title,
            content,
            version: env!("CARGO_PKG_VERSION").to_string(),
            user_email,
            is_logged_in,
        }
    }

    /*
        /// # Panics
        ///
        /// - Pannics if the `draft` is `Some` and the `updated_at` field cannot be converted to a `chrono::DateTime`.
        #[must_use]
        pub fn with_draft(
            title: String,
            content: Vec<BlogPost>,
            user_email: String,
            is_logged_in: bool,
            draft: Option<JournalDraft>,
        ) -> Self {
            let (draft_id, draft_title, draft_body, draft_author, draft_saved_at) = match draft {
                Some(draft) => {
                    let saved_at: Result<String, ()> = Ok(chrono::DateTime::<chrono::Utc>::from(
                        draft.updated_at.to_system_time(),
                    ))
                    .map(|date| date.format("%B %-d, %Y at %-I:%M:%S %p UTC").to_string());
                    (
                        draft.id.to_hex(),
                        draft.title,
                        draft.body,
                        draft.author,
                        saved_at.expect("Error with 'save_at' time"),
                    )
                }

                None => (
                    String::new(),
                    String::new(),
                    String::new(),
                    "Hunter, Christerper".to_owned(),
                    String::new(),
                ),
            };

            Self {
                title,
                version: "1".to_string(),
                content,
                user_email,
                is_logged_in,
            }
    }
        */
}

#[derive(Template, Default)]
#[template(path = "parts/posts.part.html")]
pub struct JournalFormTemplate {
    content: Vec<BlogPost>,
}

impl JournalFormTemplate {
    #[must_use]
    pub const fn new(content: Vec<BlogPost>) -> Self {
        Self { content }
    }
}

// #[derive(Template)]
// #[template(path = "parts/posts.part.html")]
// pub struct PostPart {
//     content: Vec<BlogPost>,
//     user: String,
// }

// impl PostPart {
//     #[must_use]
//     pub const fn new(content: Vec<BlogPost>, user: String) -> Self {
//         Self { content, user }
//     }
// }

#[derive(Template)]
#[template(path = "parts/draft_status.part.html")]
pub struct DraftStatusTemplate<'a> {
    pub status_class: &'a str,
    pub message: &'a str,
}

#[derive(Template)]
#[template(path = "parts/modal_load.part.html")]
pub struct ErrorPage<'a> {
    pub title: &'a str,
    pub code: u32,
    pub error: &'a str,
    pub message: &'a str,
}

impl<'a> ErrorPage<'a> {
    #[must_use]
    pub const fn new(message: &'a str) -> Self {
        Self {
            title: "Error",
            code: 500,
            error: "Internal Server Error",
            message,
        }
    }
}

#[get("/favicon")]
#[instrument(name = "Serving favicon", level = "info", target = "portfolio_site")]
pub async fn favicon() -> Result<NamedFile, actix_web::Error> {
    info!("Serving favicon");
    let filename = "head_shot.ico";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    let file = match NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[get("/logomain")]
#[instrument(name = "Serving logo", level = "info", target = "portfolio_site")]
pub async fn logomain() -> Result<NamedFile, actix_web::Error> {
    info!("Serving logo");
    let filename = "logomain.jpeg";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    let file = match NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[get("/stylesheet")]
#[instrument(name = "Serving stylesheet", level = "info", target = "portfolio_site")]
pub async fn stylesheet() -> impl Responder {
    info!("Serving stylesheet");
    let file = include_str!("../../static/css/style.css");
    HttpResponse::Ok().content_type("text/css").body(file)
}

#[get("/style.css.map")]
#[instrument(name = "Serving source map", level = "info", target = "portfolio_site")]
pub async fn source_map() -> impl Responder {
    info!("Serving source map");
    let file = include_str!("../../static/css/style.css.map");
    HttpResponse::Ok()
        .content_type("application/json")
        .body(file)
}

#[get("/htmx")]
#[instrument(
    name = "Serving htmx.min.js",
    level = "info",
    target = "portfolio_site"
)]
pub async fn htmx() -> Result<NamedFile, actix_web::Error> {
    info!("Serving htmx.min.js");

    let filename = "htmx.min.js";
    let path: PathBuf = ["static", "assets", "htmx", filename].iter().collect();
    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/response-targets")]
#[instrument(
    name = "Serving response-targets.js",
    level = "info",
    target = "portfolio_site"
)]
pub async fn response_targets() -> Result<NamedFile, actix_web::Error> {
    info!("Serving response-targets.js");

    let filename = "response-targets.js";
    let pash: PathBuf = ["static", "assets", "htmx", filename].iter().collect();
    match NamedFile::open(pash) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/sse")]
#[instrument(name = "Serving sse.js", level = "info", target = "portfolio_site")]
pub async fn sse() -> Result<NamedFile, actix_web::Error> {
    info!("Serving sse.js");

    let filename = "sse.js";
    let path: PathBuf = ["static", "assets", "htmx", filename].iter().collect();
    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/action_script")]
#[instrument(
    name = "Serving action_script.js",
    level = "info",
    target = "portfolio_site"
)]
pub async fn action_script() -> Result<NamedFile, actix_web::Error> {
    info!("Serving action_script.js");

    let filename = "action_script.js";
    let path: PathBuf = ["static", "js", filename].iter().collect();

    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/prof_headshot")]
#[instrument(
    name = "Serving prof_headshot.jpg",
    level = "info",
    target = "portfolio_site"
)]
pub async fn prof_headshot() -> Result<NamedFile, actix_web::Error> {
    info!("Serving prof_headshot.jpg");

    let filename = "head_shot.png";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/usmc_patrolling")]
#[instrument(
    name = "Serving usmc_patrolling.jpg",
    level = "info",
    target = "portfolio_site"
)]
pub async fn usmc_patrolling() -> Result<NamedFile, actix_web::Error> {
    info!("Serving usmc_patrolling.jpg");

    let filename = "usmc_patrolling.jpg";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/spinner")]
#[instrument(
    name = "Serving spinner.jpg",
    level = "info",
    target = "portfolio_site"
)]
pub async fn spinner() -> Result<NamedFile, actix_web::Error> {
    info!("Serving spinner.jpg");

    let filename = "spinner.gif";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/github")]
#[instrument(name = "Serving github.svg", level = "info", target = "portfolio_site")]
pub async fn github() -> Result<NamedFile, actix_web::Error> {
    info!("Serving github.webp");

    let filename = "github.webp";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/linkedin")]
#[instrument(
    name = "Serving linkedin.svg",
    level = "info",
    target = "portfolio_site"
)]
pub async fn linkedin() -> Result<NamedFile, actix_web::Error> {
    info!("Serving linkedin.svg");

    let filename = "linkedIn.svg";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/settings_icon")]
#[instrument(
    name = "Serving settings_icon.svg",
    level = "info",
    target = "settings_icon"
)]
pub async fn settings_icon() -> Result<NamedFile, actix_web::Error> {
    info!("Serving settings_icon");

    let filename = "gears_001.jpg";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}
