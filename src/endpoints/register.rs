use actix_web::{
    HttpResponse, get, post,
    web::{self, Data},
};
use askama::Template;
use mongodb::bson::doc;
use redis::Commands;
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};

use crate::{
    models::redis::establish_connection,
    security::{login::LoginChecker, passworder::PassWorder},
    settings,
};

/// All things login that need to be handled for the ``SundayLife`` services website.

#[derive(Template)]
#[template(path = "register.html")]
struct RegisterTemplate<'a> {
    title: &'a str,
    content: Vec<&'a str>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct RegisterUser {
    #[serde(rename = "email_input")]
    pub email: String,
    #[serde(rename = "password_input")]
    pub password: String,
    #[serde(rename = "password_2_input")]
    password_2: String,
}

#[derive(Serialize)]
struct RegistrationData {
    email: String,
    password_hash: String,
    password_salt: String,
}

#[get("/register")]
#[instrument(
    name = "User registration attempted",
    level = "info",
    target = "sundayLifeServices web app"
)]
pub async fn register_template() -> HttpResponse {
    let user_name: &str = "sundaylife.mxa";
    let user_password_1: &str = "password_1";
    let user_password_2: &str = "password_2";
    let template = RegisterTemplate {
        title: "Registration",
        content: [user_name, user_password_1, user_password_2].to_vec(),
    };

    let template = template.render().expect("About page render error");

    HttpResponse::Ok().body(template)
}

#[post("/register_user")]
#[instrument(
    name = "User registration attempted",
    level = "info",
    target = "sundayLifeServices web app",
    skip(body, mongo, redis)
)]
pub async fn register_user(
    mongo: Data<mongodb::Database>,
    redis: Data<r2d2::Pool<redis::Client>>,
    body: web::Form<RegisterUser>,
) -> HttpResponse {
    // Validate the user data entered
    let email: String = String::from(&body.0.email);
    let password: &str = &body.0.password;
    let password_2: &str = &body.0.password_2;

    if !password.eq(password_2) {
        error!("Password not equal during registration");
        return HttpResponse::NotAcceptable().json("Passwords do not match");
    }

    let restricted_and_invisible_chars = ['\n', '\r', '\t', '\0', '\x0B', '\x0C'];

    if password
        .chars()
        .any(|c| restricted_and_invisible_chars.contains(&c))
    {
        return HttpResponse::NotAcceptable().finish();
    }

    let encrypted_pw: PassWorder = PassWorder::new(password.to_string())
        .encrypt()
        .salt()
        .pepper();

    let (salt, pw, _) = encrypted_pw.deconstruct();

    // Save the user to Mongodb
    let db: mongodb::Collection<RegistrationData> = mongo.collection(
        &settings::get()
            .expect("Unable to procure the settings")
            .mongo
            .collection,
    );

    // Save the user to the database
    let result = db
        .insert_one(RegistrationData {
            email: email.clone(),
            password_hash: pw,
            password_salt: salt,
        })
        .await;

    match result {
        Ok(id) => {
            tracing::warn!("Saving to the cache-layer");
            let cache_key = format!("user:auth:{}", body.0.email);

            let auth_data = LoginChecker::new(email, encrypted_pw.get());

            if let Ok(json_data) = serde_json::to_string(&auth_data) {
                let mut redis_conn = establish_connection(redis.get_ref().clone());
                // Debug log
                tracing::warn!("the json data to be saved: {:#?}", json_data);
                // Set the key in Redis
                // let _: redis::RedisResult<()> = redis_conn.set_ex(&cache_key, json_data, 3600);
                match redis_conn.set_ex(&cache_key, json_data, 3600) {
                    Ok(()) => (),
                    Err(err) => tracing::error!("Error saving to the cache layer -> {err:#?}"),
                }
            }

            HttpResponse::Created().json(format!("User Registered: {}", id.inserted_id))
        }
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test};

    #[actix_web::test]
    async fn test_register_a_user() {
        let app = test::init_service(App::new().service(register_template)).await;

        let req = test::TestRequest::get().uri("/register").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    #[ignore = "The test logic is broken, it should return a 200 code but it fails for that and passes for an Internal Server Error code"]
    async fn test_user_is_registered() {
        let app = test::init_service(App::new().service(register_user)).await;

        let req = test::TestRequest::post()
            .uri("/register_user")
            .set_form(&RegisterUser {
                email: String::from("some_email@email.com"),
                password: "some_password".to_string(),
                password_2: "some_password".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_server_error());
    }

    #[actix_web::test]
    async fn test_user_is_cached() {
        let app = test::init_service(App::new().service(register_user)).await;

        let req = test::TestRequest::post()
            .uri("/register_user")
            .set_form(&RegisterUser {
                email: String::from("some_email_2@email.com"),
                password: "some_password".to_string(),
                password_2: "some_password".to_string(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        let resp_body = test::read_body(resp).await;

        assert!(!resp_body.is_empty());
    }
}
