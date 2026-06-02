use actix_web::{
    HttpResponse, Responder, get, post,
    web::{self, Data},
};
use askama::Template;
use mongodb::bson::{self};
use redis::Commands;
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};

use crate::{
    endpoints::register::RegisterUser,
    models::redis::establish_connection,
    security::{login::LoginChecker, passworder::PassWorder},
    settings,
};

/// All things login that need to be handled for the ``SundayLife`` services website.

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate<'a> {
    title: &'a str,
    content: Vec<&'a str>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct LoginUser {
    #[serde(rename = "email_input")]
    pub email: String,
    #[serde(rename = "password_input")]
    pub password: String,
}

impl From<RegisterUser> for LoginUser {
    fn from(value: RegisterUser) -> Self {
        Self {
            email: value.email,
            password: value.password,
        }
    }
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
    mongo: Data<mongodb::Database>,
    redis: Data<r2d2::Pool<redis::Client>>,
    body: web::Form<LoginUser>,
) -> impl Responder {
    debug!("The user data entered: {:#?}", body.0);

    // Validate the user data entered
    // let identification: ObjectId = body.0.id;
    let useremail: &str = body.0.email.as_str();
    let password: &str = body.0.password.as_str();

    let filter = mongodb::bson::doc! {
    "email":  useremail
    };

    // Check redis first
    tracing::info!("Checking the cache-layer");
    let cache_key = format!("user:auth:{useremail}");
    let mut redis_conn = establish_connection(redis.get_ref().clone());

    let cached_user: Option<String> = match redis_conn.get(cache_key) {
        Ok(cached_user) => Some(cached_user),
        Err(err) => {
            tracing::debug!("cache-miss: {err}");
            None
        }
    };

    let user_auth: LoginChecker = if let Some(json_data) = cached_user {
        tracing::warn!("cache-hit");

        let mut json_result: LoginChecker = serde_json::from_str::<LoginChecker>(&json_data)
            .expect("Unable to convert json data to LoginChecker");

        // Deconstruct the entire pw hash into the salt and pw
        let pw_hash: PassWorder = PassWorder::new(json_result.get_pw());

        let (_salt, pw, _pepper) = pw_hash.deconstruct();

        json_result.set_pw(pw);
        json_result
    } else {
        // TODO: Change this from an error to a warn
        tracing::error!("cache-miss");

        // Mongodb check of the user
        let db: mongodb::Collection<bson::Document> = mongo.collection(
            &settings::get()
                .expect("Unable to acquire settings")
                .mongo
                .collection,
        );

        let user = match db.find_one(filter).await {
            Ok(user) => bson::from_document::<LoginChecker>(user.expect("No joy"))
                .expect("Unable to convert"),
            Err(err) => {
                tracing::error!("No conversion possible from Document to LoginChecker: {err}");
                LoginChecker::default()
            }
        };

        user
    };

    if user_auth.pw_verify(password.to_string()) {
        tracing::warn!("PASSWORD VERIFIED! -> True");
        return HttpResponse::Ok().body("Login successfully");
    }

    // THIS RETURN VAL IS TEMPORARY
    tracing::error!("PASSWORD INCORRECT");
    return HttpResponse::Ok().body(format!("Invalid user entered credentials: {useremail}"));
}
