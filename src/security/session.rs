use actix_web::{
    HttpResponse,
    cookie::{Cookie, time::Duration},
};
use redis::Commands;
use uuid::Uuid;

use super::login::LoginChecker;

async fn create_session(user: &LoginChecker, redis: r2d2::Pool<redis::Client>) -> HttpResponse {
    // Generate a cryptographically strong, random session ID
    let session_id = Uuid::new_v4().to_string();
    let session_key = format!("session:{session_id}");

    // Store the session ID -> Email mapping in Redis
    // Use set_ex to define a TTL (e.g. 86400 seconds / 24 hours)
    // This is critical to prevent memory exhaustion for small caches
    let mut redis_conn = match redis.get() {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("Cache layer error: {err:#?}");
            return HttpResponse::InternalServerError().body("Cache-layer failure");
        }
    };

    match redis_conn.set_ex(&session_key, user.get_email(), 86400) {
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
        .finish();

    // Return the response with the cookie attached
    HttpResponse::Ok()
        .cookie(session_cookie)
        .body("Login successful. Session cookie set")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use redis::Commands;
    use rstest::{fixture, rstest};

    use crate::{
        security::{login::LoginChecker, session::create_session},
        settings,
    };

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
            .expect("Failed to create Redis connection pool");

        let email = "test_email@example.com";
        let user: LoginChecker = LoginChecker::new(email.to_string(), "test_password".to_string());

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
        let user: LoginChecker = LoginChecker::new(
            "some_email@example.com".to_string(),
            "some_password".to_string(),
        );

        let session_id = create_session(
            &user,
            r2d2::Pool::builder()
                .build(get_local_redis_connection)
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

        let user: LoginChecker = LoginChecker::new(
            "the_email@example.com".to_string(),
            "some_password".to_string(),
        );

        let resp = create_session(
            &user,
            r2d2::Pool::builder()
                .build(get_local_redis_connection)
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
