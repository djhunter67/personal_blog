use std::fmt::Display;

use actix_web::{
    HttpRequest, HttpResponse, post,
    web::{self, Data},
};
use chrono::DateTime;
use mongodb::{
    Collection,
    bson::{Document, doc},
};
use redis::Commands;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    models::{
        mongo,
        redis_conf::{self},
    },
    settings,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct BlogPost {
    title: String,
    body: String,
    author: String,
    email: String,
    #[serde(default)]
    date: DateTime<chrono::Utc>,
    #[serde(default)]
    logged_in: bool,
}

impl BlogPost {
    #[must_use]
    pub fn to_name() -> String {
        String::from("BlogPosts")
    }
    pub fn new(
        title: String,
        body: String,
        author: String,
        email: String,
        date: DateTime<chrono::Utc>,
        logged_in: bool,
    ) -> Self {
        let datetime = chrono::Utc::now();

        let mut date: DateTime<chrono::Utc> = date;
        // use the olderdate if the provided date is in the future
        if date > datetime {
            tracing::warn!("The provided date is in the future: {date:#?}");
            date = datetime;
        } else {
            tracing::info!("The provided date is valid: {date:#?}");
        }

        Self {
            title,
            body,
            author,
            email,
            date,
            logged_in,
        }
    }

    #[must_use]
    pub fn get_title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn get_body(&self) -> &str {
        &self.body
    }

    #[must_use]
    pub fn get_author(&self) -> &str {
        &self.author
    }

    #[must_use]
    pub const fn get_date(&self) -> &DateTime<chrono::Utc> {
        &self.date
    }

    pub const fn change_logged_in(&mut self, logged_in: bool) {
        self.logged_in = logged_in;
    }

    #[must_use]
    pub const fn is_logged_in(&self) -> bool {
        self.logged_in
    }
}

impl Display for BlogPost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Title: {}, Author: {}, Date: {}, Logged In: {}",
            self.title, self.author, self.date, self.logged_in
        )
    }
}

impl Default for BlogPost {
    fn default() -> Self {
        Self {
            title: String::new(),
            body: String::new(),
            author: String::new(),
            email: String::new(),
            logged_in: false,
            date: chrono::Utc::now(),
        }
    }
}

#[allow(clippy::future_not_send)]
#[instrument(
    name = "User submits text",
    level = "info",
    target = "Personal journal",
    skip(body, mongo, redis, req)
)]
#[post("/submit_text")]
pub async fn submit_text(
    mongo: Data<mongodb::Client>,
    redis: Data<r2d2::Pool<redis::Client>>,
    body: web::Form<BlogPost>,
    req: HttpRequest,
) -> HttpResponse {
    tracing::info!("Submit text endpoint");

    tracing::warn!("body: {body:#?}");
    // Check to see if the user is logged in
    match validate_user(&req, &redis) {
        Ok(user_exists) if !user_exists => {
            return HttpResponse::Ok().body("NOT LOGGED IN");
        }
        Ok(user_exists) => {
            tracing::info!("User is logged in: {user_exists}");
            let mut post = body.into_inner();
            post.change_logged_in(user_exists);

            // Save the post to the database
            let db: Collection<Document> =
                match mongo::establish_connection(mongo.get_ref().clone()).await {
                    Ok(db) => db,
                    Err(err) => {
                        tracing::error!("Error establishing connection to database: {err:#?}");
                        return HttpResponse::InternalServerError().body(format!(
                            "Error establishing connection to database: {err:#?}"
                        ));
                    }
                }
                .collection(&BlogPost::to_name());

            tracing::info!("Post: {:#?}", post.get_body());

            let oid = db
                .insert_one(doc! {
                "title": post.get_title(),
                "body": post.get_body(),
                "author": post.get_author(),
                "date": post.get_date().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                "logged_in": post.is_logged_in(),
                })
                .await
                .expect("Unable to insert post into database");

            // Save the object ID to Redis with the user's session ID as the key
            let session_id = req
                .cookie("session_id")
                .expect("No session cookie found")
                .value()
                .to_string();

            let mut red_conn = match redis_conf::establish_connection(redis.get_ref().clone()) {
                Ok(conn) => conn,
                Err(err) => {
                    tracing::error!("Error establishing connection to Redis: {err:#?}");
                    return HttpResponse::InternalServerError()
                        .body(format!("Error establishing connection to Redis: {err:#?}"));
                }
            };

            let cache_key = format!("blog_post:{session_id}");

            red_conn
                .set::<String, String, ()>(cache_key, oid.inserted_id.to_string())
                .expect("Unable to set session ID in Redis");

            return HttpResponse::Ok().body(format!("{}", oid.inserted_id));
        }

        Err(err) => {
            tracing::error!("Error validating user: {err:#?}");
            return HttpResponse::Unauthorized().body(format!("{err:#?}"));
        }
    }
}

/// # Errors
///   If the user is not logged in, return an "`HttpResponse::Unauthorized`" error.
/// # Panics
///   If the "`redis`" connection pool is not available, the function will panic.
pub fn validate_user(
    req: &HttpRequest,
    redis: &Data<r2d2::Pool<redis::Client>>,
) -> Result<bool, HttpResponse> {
    let session_id = if let Some(cookie) = req.cookie("session_id") {
        tracing::info!("Cookie found");
        cookie.value().to_string()
    } else {
        tracing::error!("User cookie not found: {:#?}", req.cookies());
        return Err(HttpResponse::Unauthorized().body(format!(
            "No session found: {:#?}",
            req.cookies().expect("No cookies found")
        )));
    };

    let mut red_conn = match redis_conf::establish_connection(redis.get_ref().clone()) {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("Unable to acquire the cache layer connection: {err:#?}");
            return Err(
                HttpResponse::InternalServerError().body(format!("Cache layer error: {err:#?}"))
            );
        }
    };

    tracing::info!("Creating the session key");
    let session_key = format!(
        "{}{}",
        &settings::get()
            .expect("Unable to procure the app settings")
            .redis
            .key,
        session_id
    );

    tracing::info!("Searching for the session key: {session_key}");
    let user = match redis::cmd("GET")
        .arg(&session_key)
        .query::<Option<String>>(&mut red_conn)
    {
        Ok(result) => result,
        Err(err) => {
            tracing::error!("Error accessing the cache layer: {err:#?}");
            return Err(HttpResponse::InternalServerError()
                .body(format!("Unable to acquire the cache layer: {err:#?}")));
        }
    };

    if user.is_none() {
        tracing::warn!("User is not logged in: {user:#?}");
        Ok(false)
    } else {
        tracing::info!("User is logged in: {user:#?}");
        Ok(true)
    }
}
