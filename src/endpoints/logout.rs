use actix_web::{HttpRequest, HttpResponse, Responder, get, web::Data};
use askama::Template;
use redis::Commands;
use tracing::instrument;

use crate::{endpoints::templates::IndexTemplate, models::redis_conf};

#[allow(clippy::future_not_send)]
#[get("/logout")]
#[instrument(
    name = "User logout attempted",
    level = "info",
    target = "sundayLifeServices web app",
    skip(redis_client, req)
)]
pub async fn logout(
    redis_client: Data<r2d2::Pool<redis::Client>>,
    req: HttpRequest,
) -> impl Responder {
    // extract the session key from the frontend

    tracing::info!("Connecting to the cache layer to remove the session key");
    let mut redis_conn = match redis_conf::establish_connection(&redis_client) {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("Unable to procure the cache-layer connection: {err:#?}");
            return HttpResponse::InternalServerError()
                .body(format!("Unable to procure the cache layer: {err:#?}"));
        }
    };

    let session_id = if let Some(cookie) = req.cookie("session_id") {
        cookie.value().to_string()
    } else {
        tracing::error!("User cookie not found: {req:#?}");
        return HttpResponse::Unauthorized().body(format!(
            "No session found: {:#?}",
            req.cookies().expect("No cookies found")
        ));
    };

    let cache_key = format!("session:{session_id}");
    let _: () = match redis_conn.del::<String, ()>(cache_key) {
        Ok(()) => {
            tracing::info!("Successfully removed the session key");
            ()
        }
        Err(err) => {
            tracing::error!("Unable to delete the session data from the cache-layer: {err:#?}");
            return HttpResponse::InternalServerError().body(format!(
                "Unable to delete the session data from the cache layer: {err:#?}"
            ));
        }
    };

    let index_template =
        IndexTemplate::new("Logged out".to_string(), vec![], "None".to_string(), false);

    HttpResponse::Ok().body(
        index_template
            .render()
            .expect("Failed to render the home page"),
    )
}
