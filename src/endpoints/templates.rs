use actix_files::NamedFile;
use actix_web::{HttpResponse, Responder, get};
use askama::Template;
use std::path::PathBuf;
use tracing::instrument;

use crate::{personnel::users, startup::VERSION};

use super::user_input::BlogPost;

/// # TODO
///
/// Dark Theme
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: String,
    pub content: Vec<BlogPost>,
    pub version: String,
    pub user_email: String,
    pub is_logged_in: bool,
}

fn default_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

impl Default for IndexTemplate {
    fn default() -> Self {
        Self {
            title: String::from("Home"),
            content: Vec::default(),
            version: default_version(),
            user_email: String::from("Please login to create a journal entry"),
            is_logged_in: false,
        }
    }
}

impl IndexTemplate {
    /// Creates a new [`IndexTemplate`].
    #[must_use]
    pub fn new(content: Vec<BlogPost>, user_email: &str, is_logged_in: bool) -> Self {
        Self {
            title: String::from("Home"),
            content,
            version: env!("CARGO_PKG_VERSION").to_string(),
            user_email: String::from(user_email),
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

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate<'a> {
    pub title: &'a str,
    pub is_logged_in: bool,
    pub user_email: &'a str,
    pub version: &'a str,
}

impl Default for LoginTemplate<'_> {
    fn default() -> Self {
        Self {
            title: "Login",
            is_logged_in: Default::default(),
            user_email: "Please login to create a post",
            version: VERSION,
        }
    }
}

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate<'a> {
    pub title: &'a str,
    pub content: Vec<&'a str>,
    pub is_logged_in: bool,
    pub user_email: &'a str,
    pub version: &'a str,
}

impl Default for RegisterTemplate<'_> {
    fn default() -> Self {
        Self {
            title: Default::default(),
            content: Vec::default(),
            is_logged_in: Default::default(),
            user_email: Default::default(),
            version: VERSION,
        }
    }
}

#[derive(Template)]
#[template(path = "settings.html")]
pub struct SettingsTemplate<'a> {
    pub title: &'a str,
    pub is_logged_in: bool,
    pub user_email: &'a str,
    pub version: &'a str,
}

impl Default for SettingsTemplate<'_> {
    fn default() -> Self {
        Self {
            title: Default::default(),
            is_logged_in: Default::default(),
            user_email: Default::default(),
            version: VERSION,
        }
    }
}

#[derive(Template)]
#[template(path = "parts/indiv_post.html")]
pub struct IndivPost {
    content: BlogPost,
}

impl IndivPost {
    #[must_use = "Create a new IndivPost"]
    pub const fn new(content: BlogPost) -> Self {
        Self { content }
    }
}

#[derive(Template)]
#[template(path = "parts/journal_form.part.html")]
pub struct IndivInput {
    content: users::Users,
}
impl IndivInput {
    #[must_use = "Create a new instance since the members are private"]
    pub const fn new(content: users::Users) -> Self {
        Self { content }
    }
}

#[derive(Template)]
#[template(path = "parts/journal_form_input.part.html")]
pub struct JournalPostEditor {
    content: BlogPost,
}

impl JournalPostEditor {
    #[must_use = "This function is used to allow a user to edit their post"]
    pub const fn new(content: BlogPost) -> Self {
        Self { content }
    }
}

#[derive(Template)]
#[template(path = "parts/confirmations.part.html")]
pub struct Confirmation {
    header_message: String,
    body_message: String,
}

impl Confirmation {
    #[must_use]
    pub const fn new(header_message: String, body_message: String) -> Self {
        Self {
            header_message,
            body_message,
        }
    }
}

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
#[instrument(name = "Serving favicon", level = "info", target = "Static Content")]
pub async fn favicon() -> Result<NamedFile, actix_web::Error> {
    tracing::info!("Serving favicon");
    let filename = "head_shot.ico";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    let file = match NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[get("/icon-192")]
#[instrument(
    name = "Serving the icons-192",
    level = "info",
    target = "Static Content"
)]
pub async fn icon_192() -> Result<NamedFile, actix_web::Error> {
    tracing::info!("Serving icon-192");
    let filename = "icon-192.png";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    let file = match NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[get("/icon-512")]
#[instrument(
    name = "Serving the icons-512",
    level = "info",
    target = "Static Content"
)]
pub async fn icon_512() -> Result<NamedFile, actix_web::Error> {
    tracing::info!("Serving icon-512");
    let filename = "icon-512.png";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    let file = match NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[get("/icon_large")]
#[instrument(
    name = "Serving the icon_large",
    level = "info",
    target = "Static Content"
)]
pub async fn icon_large() -> Result<NamedFile, actix_web::Error> {
    tracing::info!("Serving icon_large");
    let filename = "icon_large.png";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    let file = match NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[get("/link_preview")]
#[instrument(
    name = "Serving the icon_large",
    level = "info",
    target = "Static Content"
)]
pub async fn link_preview() -> Result<NamedFile, actix_web::Error> {
    tracing::info!("Serving the link preview");
    let filename = "icon_1200x627.png";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    let file = match NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[get("/manifest.webmanifest")]
#[instrument(
    name = "Serving the manifest",
    level = "info",
    target = "Static Content"
)]
pub async fn manifest() -> Result<NamedFile, actix_web::Error> {
    tracing::info!("Serving manifest");
    let filename = "manifest.json";
    let path: PathBuf = ["static", filename].iter().collect();

    let file = match NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[get("/logomain")]
#[instrument(name = "Serving logo", level = "info", target = "Static Content")]
pub async fn logomain() -> Result<NamedFile, actix_web::Error> {
    tracing::info!("Serving logo");
    let filename = "logomain.jpeg";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    let file = match NamedFile::open(path) {
        Ok(file) => file,
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            return Err(actix_web::error::ErrorInternalServerError(err));
        }
    };

    Ok(file)
}

#[get("/stylesheet")]
#[instrument(name = "Serving stylesheet", level = "info", target = "Static Content")]
pub async fn stylesheet() -> impl Responder {
    tracing::info!("Serving stylesheet");
    let file = include_str!("../../static/css/style.css");
    HttpResponse::Ok().content_type("text/css").body(file)
}

#[get("/style.css.map")]
#[instrument(name = "Serving source map", level = "info", target = "Static Content")]
pub async fn source_map() -> impl Responder {
    tracing::info!("Serving source map");
    let file = include_str!("../../static/css/style.css.map");
    HttpResponse::Ok()
        .content_type("application/json")
        .body(file)
}

#[get("/htmx")]
#[instrument(
    name = "Serving htmx.min.js",
    level = "info",
    target = "Static Content"
)]
pub async fn htmx() -> Result<NamedFile, actix_web::Error> {
    tracing::info!("Serving htmx.min.js");

    let filename = "htmx.min.js";
    let path: PathBuf = ["static", "assets", "htmx", filename].iter().collect();
    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/response-targets")]
#[instrument(
    name = "Serving response-targets.js",
    level = "info",
    target = "Static Content"
)]
pub async fn response_targets() -> Result<NamedFile, actix_web::Error> {
    tracing::info!("Serving response-targets.js");

    let filename = "response-targets.js";
    let pash: PathBuf = ["static", "assets", "htmx", filename].iter().collect();
    match NamedFile::open(pash) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/sse")]
#[instrument(name = "Serving sse.js", level = "info", target = "Static Content")]
pub async fn sse() -> Result<NamedFile, actix_web::Error> {
    tracing::info!("Serving sse.js");

    let filename = "sse.js";
    let path: PathBuf = ["static", "assets", "htmx", filename].iter().collect();
    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/action_script")]
#[instrument(
    name = "Serving action_script.js",
    level = "info",
    target = "Static Content"
)]
pub async fn action_script() -> Result<NamedFile, actix_web::Error> {
    tracing::info!("Serving action_script.js");

    let filename = "action_script.js";
    let path: PathBuf = ["static", "js", filename].iter().collect();

    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/prof_headshot")]
#[instrument(
    name = "Serving prof_headshot.jpg",
    level = "info",
    target = "Static Content"
)]
pub async fn prof_headshot() -> Result<NamedFile, actix_web::Error> {
    tracing::info!("Serving prof_headshot.jpg");

    let filename = "head_shot.png";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/usmc_patrolling")]
#[instrument(
    name = "Serving usmc_patrolling.jpg",
    level = "info",
    target = "Static Content"
)]
pub async fn usmc_patrolling() -> Result<NamedFile, actix_web::Error> {
    tracing::info!("Serving usmc_patrolling.jpg");

    let filename = "usmc_patrolling.jpg";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/spinner")]
#[instrument(
    name = "Serving spinner.jpg",
    level = "info",
    target = "Static Content"
)]
pub async fn spinner() -> Result<NamedFile, actix_web::Error> {
    tracing::info!("Serving spinner.jpg");

    let filename = "spinner.gif";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/github")]
#[instrument(name = "Serving github.svg", level = "info", target = "Static Content")]
pub async fn github() -> Result<NamedFile, actix_web::Error> {
    tracing::info!("Serving github.webp");

    let filename = "github.webp";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}

#[get("/linkedin")]
#[instrument(
    name = "Serving linkedin.svg",
    level = "info",
    target = "Static Content"
)]
pub async fn linkedin() -> Result<NamedFile, actix_web::Error> {
    tracing::info!("Serving linkedin.svg");

    let filename = "linkedIn.svg";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
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
    tracing::info!("Serving settings_icon");

    let filename = "gears_001.jpg";
    let path: PathBuf = ["static", "imgs", filename].iter().collect();

    match NamedFile::open(path) {
        Ok(file) => Ok(file),
        Err(err) => {
            tracing::error!("Error opening file -- {filename} -- : {err:#?}");
            Err(actix_web::error::ErrorInternalServerError(err))
        }
    }
}
