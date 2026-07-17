use actix_web::{
    HttpResponse, get,
    web::{self, Data},
};
use askama::Template;
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Template)]
#[template(path = "settings.html")]
struct SettingsTemplate<'a> {
    title: &'a str,
    user: &'a str,
    is_logged_in: bool,
}

#[derive(Deserialize, Serialize)]
pub struct UserSettingsChange<'a> {
    #[serde(rename = "email_input")]
    email: &'a str,
    #[serde(rename = "current_password")]
    orig_pw: &'a str,
    #[serde(rename = "new_password")]
    new_pw: &'a str,
    #[serde(rename = "new_password_2")]
    new_pw_2: &'a str,
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

#[must_use = "The user would like to change some things"]
pub fn settings_change(
    _mongo: Data<mongodb::Client>,
    _redis: Data<r2d2::Pool<redis::Client>>,
    body: &web::Form<UserSettingsChange<'static>>,
) -> HttpResponse {
    // let user_email = body.email;
    // let orig_pw = body.orig_pw;
    let new_pw = body.new_pw;
    let new_pw_2 = body.new_pw_2;

    if !new_pw.eq(new_pw_2) {
        return HttpResponse::BadRequest().body("Passwords do not match");
    }

    HttpResponse::Ok().into()
}

mod data_and_accounts {
    use chrono::{DateTime, Utc};
    use mongodb::bson::Uuid;

    /// # TODO
    ///
    /// Forgotten password recovery
    /// Email address verification
    /// New login alerts
    /// Password change alerts
    /// Suspicious activity notifications
    /// Account deletion confirmation
    /// Email based two-factor authentication
    pub struct _AccountEvent<'a> {
        user_id: Uuid,
        occurred_at: DateTime<Utc>,
        ip_prefix: Option<&'a str>,
        user_agent_summary: Option<&'a str>,
        metadata: serde_json::Value,
    }
}
