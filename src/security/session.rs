use actix_web::{
    HttpResponse,
    cookie::{Cookie, time::Duration},
};
use askama::Template;
use redis::Commands;
use uuid::Uuid;

use crate::{endpoints::login::LoginTemplate, personnel::users};

/// # Panics
///
/// If the cookie cannot be built, the function will panic.
pub async fn create_session(
    user: &users::Users,
    mut redis: r2d2::PooledConnection<redis::Client>,
) -> HttpResponse {
    tracing::info!("Generating the cookie");
    // Generate a cryptographically strong, random session ID
    let session_id = Uuid::new_v4().to_string();
    let session_key = format!("session:{session_id}");

    match redis.set_ex(&session_key, user.get_email(), 86400) {
        Ok(()) => (),
        Err(err) => {
            tracing::error!("Unable to set the session key into the cache layer: {err:#?}");
            return HttpResponse::InternalServerError().body(err.to_string());
        }
    }

    // Build the HTTP-only, Secure cookie
    let session_cookie: Cookie = Cookie::build("session_id", session_id)
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(actix_web::cookie::SameSite::Strict)
        .max_age(Duration::seconds(86400))
        .expires(actix_web::cookie::time::OffsetDateTime::now_utc() + Duration::seconds(86400))
        .finish();

    let template = LoginTemplate {
        user_email: &user.get_email(),
        ..Default::default()
    };

    let render = template.render().expect("unable to render web page");

    tracing::info!("The session cookie to insert: {session_cookie}");

    HttpResponse::Ok().cookie(session_cookie).body(render)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use redis::Commands;
    use rstest::{fixture, rstest};

    use crate::{personnel::users, security::session::create_session, settings};

    #[fixture]
    fn get_local_redis_connection() -> redis::Client {
        redis::Client::open(settings::get().unwrap().redis.uri)
            .expect("Failed to create Redis client")
    }

    #[rstest]
    #[actix_web::test]
    async fn test_create_session_sets_cookie(get_local_redis_connection: redis::Client) {
        let conn = r2d2::Pool::builder()
            .max_size(15)
            .build(get_local_redis_connection)
            .expect("Failed to create Redis connection pool")
            .get()
            .unwrap();

        let email = "test_email@example.com";
        let user: users::Users = users::Users::new(
            email.to_string(),
            "test_password".to_string(),
            String::new(),
        );

        let resp = create_session(&user, conn).await;

        // Check that the response has a Set-Cookie header
        let cookies = resp.cookies().collect::<Vec<_>>();
        assert_eq!(cookies.len(), 1);
        let cookie = &cookies[0];
        assert_eq!(cookie.name(), "session_id");
        assert!(cookie.http_only().unwrap());
        assert!(cookie.secure().unwrap());
        assert_eq!(cookie.path().unwrap(), "/");
    }

    #[rstest]
    #[actix_web::test]
    async fn test_create_session_stores_in_redis(get_local_redis_connection: redis::Client) {
        let mut conn = get_local_redis_connection.get_connection().unwrap();
        let user: users::Users = users::Users::new(
            "some_email@example.com".to_string(),
            "some_password".to_string(),
            String::new(),
        );

        let session_id = create_session(
            &user,
            r2d2::Pool::builder()
                .build(get_local_redis_connection)
                .unwrap()
                .get()
                .unwrap(),
        )
        .await
        .cookies()
        .find(|cookie| cookie.name() == "session_id")
        .unwrap()
        .value()
        .to_string();

        let session_key = format!("session:{session_id}");
        let stored_email: String = conn.get(&session_key).unwrap();
        assert_eq!(stored_email, user.get_email());
    }

    #[rstest]
    #[actix_web::test]
    async fn test_create_session_has_ttl(get_local_redis_connection: redis::Client) {
        let mut conn = get_local_redis_connection.get_connection().unwrap();

        let user: users::Users = users::Users::new(
            "the_email@example.com".to_string(),
            "some_password".to_string(),
            String::new(),
        );

        let resp = create_session(
            &user,
            r2d2::Pool::builder()
                .build(get_local_redis_connection)
                .unwrap()
                .get()
                .unwrap(),
        )
        .await;

        let session_id = resp
            .cookies()
            .find(|cookie| cookie.name() == "session_id")
            .unwrap()
            .value()
            .to_string();

        let session_key = format!("session:{session_id}");
        let ttl: i64 = conn.ttl(&session_key).unwrap();

        // Check that the TTL is set (greater than 0 and less than or equal to (86400 seconds / 24 hours))
        assert!(ttl > 0 && ttl <= 86400);
    }
}
