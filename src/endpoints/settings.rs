use std::collections::HashMap;

use actix_multipart::form::{MultipartForm, tempfile::TempFile, text::Text};
use actix_web::{HttpResponse, get, post, web::Data};
use askama::Template;
use tracing::instrument;

#[derive(Template)]
#[template(path = "settings.html")]
struct SettingsTemplate<'a> {
    title: &'a str,
    user: &'a str,
    is_logged_in: bool,
}

#[derive(Debug, MultipartForm)]
pub struct UserSettingsChange {
    #[multipart(rename = "email_input")]
    pub email: Option<Text<String>>,
    #[multipart(rename = "current_password")]
    pub orig_pw: Option<Text<String>>,
    #[multipart(rename = "new_password")]
    pub new_pw: Option<Text<String>>,
    #[multipart(rename = "new_password_2")]
    pub new_pw_2: Option<Text<String>>,
    #[multipart(rename = "profile_image")]
    pub image: Option<TempFile>,
}

#[get("/settings")]
#[instrument(
    name = "User settings",
    level = "info",
    target = "personal journal web app"
)]
pub async fn settings_template() -> HttpResponse {
    tracing::info!("Login page loaded");

    let template = SettingsTemplate {
        title: "Settings",
        user: "logged in user",
        is_logged_in: true,
    };

    let template = template.render().expect("Login page render error");

    HttpResponse::Ok().body(template)
}

/// All input are optional
#[post("/settings_change")]
#[instrument(
    name = "User settings change",
    level = "info",
    target = "personal journal web app",
    skip(_mongo, _redis, body)
)]
pub async fn settings_change(
    _mongo: Data<mongodb::Client>,
    _redis: Data<r2d2::Pool<redis::Client>>,
    MultipartForm(body): MultipartForm<UserSettingsChange>,
) -> HttpResponse {
    let pw_1 = body.new_pw.as_ref().map(|pw| pw.as_str());

    let pw_2 = body.new_pw_2.as_ref().map(|pw| pw.as_str());

    if let Some(img) = &body.image {
        tracing::warn!(
            size = img.size / 1024,
            file_name = ?img.file_name,
            content_type = ?img.content_type,
        );
    } else {
        tracing::warn!("No image uploaded");
    }

    // tracing::warn the size of the vector holding the image bytes
    if let Some(img) = &body.image {
        let img_bytes = img.size / 1024;
        tracing::warn!("Image bytes size: {} MB", img_bytes / 100);
    }

    if !pw_1.eq(&pw_2) {
        // return HttpResponse::BadRequest().body("Passwords do not match");
        return HttpResponse::Ok().body("Passwords do not match");
    }
    // tracing::warn!("Body: {:#?}", body);

    HttpResponse::Ok().json(HashMap::from([
        (
            "email",
            body.email
                .map(actix_multipart::form::text::Text::into_inner),
        ),
        (
            "orig_pw",
            body.orig_pw
                .map(actix_multipart::form::text::Text::into_inner),
        ),
        (
            "new_pw",
            body.new_pw
                .map(actix_multipart::form::text::Text::into_inner),
        ),
        (
            "new_pw_2",
            body.new_pw_2
                .map(actix_multipart::form::text::Text::into_inner),
        ),
    ]))
}
