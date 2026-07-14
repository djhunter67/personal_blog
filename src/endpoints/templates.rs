use actix_files::NamedFile;
use actix_web::{HttpResponse, Responder, get};
use askama::Template;
use std::path::PathBuf;
use tracing::{error, info, instrument};

use super::user_input::BlogPost;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub title: &'a str,
    pub content: Vec<BlogPost>,
    pub version: &'a str,
    pub user: &'a str,
}

impl<'a> IndexTemplate<'a> {
    #[must_use]
    pub const fn new(title: &'a str, content: Vec<BlogPost>, user: &'a str) -> Self {
        Self {
            title,
            content,
            version: env!("CARGO_PKG_VERSION"),
            user,
        }
    }
}

#[derive(Template)]
#[template(path = "parts/posts.part.html")]
pub struct PostPart {
    content: Vec<BlogPost>,
    user: String,
}

impl PostPart {
    #[must_use]
    pub const fn new(content: Vec<BlogPost>, user: String) -> Self {
        Self { content, user }
    }
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
