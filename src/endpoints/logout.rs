use actix_web::{HttpRequest, HttpResponse, Responder, get, web::Data};
use askama::Template;
use redis::{AsyncCommands, aio};
use tracing::instrument;

use crate::endpoints::templates::IndexTemplate;

#[allow(clippy::future_not_send)]
#[get("/logout")]
#[instrument(
    name = "User logout attempted",
    level = "info",
    target = "sundayLifeServices web app",
    skip(redis_client, req)
)]
pub async fn logout(
    redis_client: Data<aio::ConnectionManager>,
    req: HttpRequest,
) -> impl Responder {
    // extract the session key from the frontend

    tracing::info!("Connecting to the cache layer to remove the session key");

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
    let _: () = match redis_client
        .as_ref()
        .clone()
        .del::<String, ()>(cache_key)
        .await
    {
        Ok(()) => {
            tracing::info!("Successfully removed the session key");
        }
        Err(err) => {
            tracing::error!("Unable to delete the session data from the cache-layer: {err:#?}");
            return HttpResponse::InternalServerError().body(format!(
                "Unable to delete the session data from the cache layer: {err:#?}"
            ));
        }
    };

    let index_template =
        IndexTemplate::new(vec![], "Please login to create a journal entry", false);

    HttpResponse::Ok().body(
        index_template
            .render()
            .expect("Failed to render the home page"),
    )
}
