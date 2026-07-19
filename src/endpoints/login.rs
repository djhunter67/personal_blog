use actix_web::{
    HttpResponse, Responder, get, post,
    web::{self, Data},
};
use askama::Template;
use redis::Commands;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::{
    models::{mongo, redis_conf},
    security::{login::LoginChecker, passworder::PassWorder, session::create_session},
    settings,
};

/// All things login that need to be handled for the ``SundayLife`` services website.

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate<'a> {
    pub title: &'a str,
    pub content: Vec<&'a str>,
    pub user_id: &'a str,
    pub is_logged_in: bool,
}

/// # TODO:
///
/// Activity timestamps
/// Usage analytics
/// Retain login history
/// Delete user account
/// Successful logins
/// Failed login attempts
/// Password changed before
/// Data exported
#[derive(Deserialize, Debug, Serialize)]
pub struct LoginUser {
    #[serde(rename = "email_input")]
    pub email: String,
    #[serde(rename = "password_input")]
    pub password: String,
}

#[get("/login")]
#[instrument(
    name = "User login attempted",
    level = "info",
    target = "sundayLifeServices web app"
)]
pub async fn login_template() -> HttpResponse {
    debug!("Login page loaded");

    let user_login: &str = "user_email";
    let user_password: &str = "super_duper_secret_password";
    let template = LoginTemplate {
        title: "Login",
        content: [user_login, user_password].to_vec(),
        user_id: "logged in user",
        is_logged_in: false,
    };

    let template = template.render().expect("Login page render error");

    HttpResponse::Ok().body(template)
}

#[post("/login_user")]
#[instrument(
    name = "User login attempted",
    level = "info",
    target = "sundayLifeServices web app",
    skip(body, mongo, redis)
)]
pub async fn login_user(
    mongo: Data<mongodb::Client>,
    redis: Data<r2d2::Pool<redis::Client>>,
    body: web::Form<LoginUser>,
) -> impl Responder {
    debug!("The user data entered: {:#?}", body.0);

    // Validate the user data entered
    let useremail: &str = body.0.email.as_str();
    let password: &str = body.0.password.as_str();

    let filter = mongodb::bson::doc! {
    "email":  useremail
    };

    // Check redis first
    tracing::info!("Checking the cache-layer");
    let cache_key = format!("user:auth:{useremail}");
    let mut redis_conn = match redis_conf::establish_connection(&redis) {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("Unable to procure the cache-layer connection: {err:#?}");
            return HttpResponse::InternalServerError()
                .body(format!("Unable to procure the cache layer: {err:#?}"));
        }
    };

    let cached_user: Option<String> = match redis_conn.get(cache_key) {
        Ok(cached_user) => Some(cached_user),
        Err(err) => {
            tracing::debug!("cache-miss: {err}");
            None
        }
    };

    let user_auth: LoginChecker = if let Some(json_data) = cached_user {
        // redundant Option to satisfy the compiler
        tracing::warn!("cache-hit: {json_data:#?}");

        let mut json_result = LoginChecker::new(useremail.to_string(), password.to_string());

        // let mut json_result: LoginChecker = match serde_json::from_str::<LoginChecker>(&json_data) {
        //     Ok(json_result) => json_result,
        //     Err(err) => {
        //         tracing::error!("Unable to convert json data to LoginChecker: {err:#?}");
        //         LoginChecker::default()
        //     }
        // };
        // .expect("Unable to convert json data to LoginChecker");

        // Deconstruct the entire pw hash into the salt and pw
        let pw_hash: PassWorder = PassWorder::new(json_result.get_pw())
            .encrypt()
            .salt()
            .pepper();

        let (_salt, pw, _pepper) = pw_hash.deconstruct();

        json_result.set_pw(pw);
        json_result
    } else {
        // TODO: Change this from an error to a warn
        tracing::error!("cache-miss");

        let db: mongodb::Collection<LoginChecker> = match mongo::establish_connection(&mongo).await
        {
            Ok(db) => db,
            Err(err) => {
                tracing::error!("Unable to procure the database: {err:#?}");
                return HttpResponse::InternalServerError()
                    .body(format!("Unable to procure the database: {err:#?}"));
            }
        }
        .collection(
            &match settings::get() {
                Ok(settings) => settings,
                Err(err) => {
                    tracing::error!("Unable to procure database settings: {err:#?}");
                    return HttpResponse::InternalServerError()
                        .body(format!("Unable to procure the database settings: {err:#?}"));
                }
            }
            .mongo
            .collection,
        );

        match db.find_one(filter).await {
            Ok(user) => {
                if let Some(user_found) = user {
                    user_found
                } else {
                    tracing::error!("No user data found in the database");
                    // Change this to unauthorized
                    return HttpResponse::Ok().body("No user data matching the supplied email");
                }
            }
            Err(err) => {
                tracing::error!("No conversion possible from Document to LoginChecker: {err}");
                LoginChecker::default()
            }
        }
    };

    if user_auth.pw_verify(password.to_string()) {
        tracing::warn!("PASSWORD VERIFIED! -> True");
        return create_session(&user_auth, redis_conn).await;
    }

    // THIS RETURN VAL IS TEMPORARY
    tracing::error!("PASSWORD INCORRECT");
    return HttpResponse::Ok().body(format!("Invalid user entered credentials: {useremail}"));
}
