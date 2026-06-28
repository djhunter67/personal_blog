use std::task::Poll;

// use crate::{endpoints::templates::ErrorPage, models::redis_conf::establish_connection, settings};

use super::templates::IndexTemplate;
use actix_web::{
    Error, HttpRequest, HttpResponse, Responder, get,
    http::{
        self, StatusCode,
        header::{ContentEncoding, ContentType},
    },
    web::{self, Data},
};
use askama::Template;
use futures::stream;
use tracing::{info, instrument};

#[allow(clippy::future_not_send)]
#[instrument(
    name = "Serving main page",
    level = "debug",
    target = "web_app_bloodhound",
    fields(samples = 25, title = "Home"),
    skip(_redis, _req)
)]
#[get("/")]
pub async fn index(_req: HttpRequest, _redis: Data<r2d2::Pool<redis::Client>>) -> HttpResponse {
    info!("Serving main page");

    tracing::info!("About page loading");
    //     let session_id = if let Some(cookie) = req.cookie("session_id") {
    //         cookie.value().to_string()
    //     } else {
    //         tracing::error!("User cookie not found: {req:#?}");

    //         let var_name = IndexTemplate {
    //         title: "Home",
    //         content: [".lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
    // ", ".lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
    // ",".lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
    // "].to_vec(),
    //         version: "0.1.0",
    // 	user: "NEW USER"
    //     };

    //         let rendered = var_name.render().expect("Failed to render template");

    //         // return HttpResponse::Unauthorized().body(rendered);
    //         return HttpResponse::Ok().body(rendered);
    //     };

    //     let mut red_conn = match establish_connection(redis.get_ref().clone()) {
    //         Ok(conn) => conn,
    //         Err(err) => {
    //             tracing::error!("Unable to acquire the redis connection: {err:#?}");
    //             return HttpResponse::InternalServerError()
    //                 .body(format!("Cache layer error: {err:#?}"));
    //         }
    //     };

    //     tracing::info!("Creating the session key");
    //     let session_key = format!(
    //         "{}{}",
    //         &settings::get()
    //             .expect("Unable to procure the app settings")
    //             .redis
    //             .key,
    //         session_id
    //     );

    //     tracing::info!("Searching for the session key: {session_key}");
    //     let user = match redis::cmd("GET")
    //         .arg(&session_key)
    //         .query::<Option<String>>(&mut red_conn)
    //     {
    //         Ok(result) => result,
    //         Err(err) => {
    //             tracing::error!("Error accessing the cache layer: {err:#?}");
    //             return HttpResponse::InternalServerError()
    //                 .body(format!("Unable to acquire the cache layer: {err:#?}"));
    //         }
    //     };

    //     tracing::warn!("The session id: {session_key}");
    //     user.map_or_else(
    //         || {
    // 	    tracing::error!("Unable to procure the user based on the session key");
    // 	    let error_template = ErrorPage {
    // 		title: "Cache-Error",
    // 		code: 500,
    // 		error: "Session key live but no user data associated with the session key",
    // 		message: "Logout, if possible, and log back in"
    // 	    };

    // 	    let rendered = error_template.render().expect("unable to render the error template");
    // 	    HttpResponse::Unauthorized().body(rendered)
    // 	},
    //         |email| {

    let email = String::from("some_email@email.com");

    let var_name = IndexTemplate::new(
        "Home",
         [".lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
", ".lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
",".lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
"].to_vec(),
	 &email
    );

    let rendered = var_name.render().expect("Failed to render template");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(rendered)
    // },
    // )
}

#[allow(clippy::future_not_send)]
pub async fn sse(_req: HttpRequest) -> impl Responder {
    let mut counter: usize = 5;

    // yeilds `data N` whrere N in [5; 1]
    let server_events = stream::poll_fn(move |_cx| -> Poll<Option<Result<web::Bytes, Error>>> {
        if counter == 0 {
            return Poll::Ready(None);
        }
        let payload = format!("data: {counter}\n\n");
        counter -= 1;
        Poll::Ready(Some(Ok(web::Bytes::from(payload))))
    });

    HttpResponse::build(StatusCode::OK)
        .insert_header((http::header::CONTENT_TYPE, "text/event-stream"))
        .insert_header(ContentEncoding::Identity)
        .streaming(server_events)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::pin::pin;

    use actix_web::{
        App,
        body::{self, MessageBody},
        test,
        web::{self, Bytes},
    };
    use futures::future;

    use super::{index, sse};

    #[actix_web::test]
    #[ignore = "known to fail in this stage of development"]
    async fn test_get_index() {
        let app = test::init_service(App::new().service(index)).await;
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    #[ignore = "known to fail during this stage of development"]
    async fn test_index_is_html() {
        let app = test::init_service(App::new().service(index)).await;
        let req = test::TestRequest::get().uri("/").to_request();

        let resp = test::call_and_read_body(&app, req).await;
        assert!(!resp.is_empty());
        let first_letters: Bytes = resp.slice(0..15).iter().copied().collect();
        let conv_str = std::str::from_utf8(&first_letters).unwrap();
        assert_eq!(conv_str, "<!DOCTYPE html>");
    }

    #[actix_web::test]
    async fn test_stream_chunk() {
        let app = test::init_service(App::new().route("/sse", web::get().to(sse))).await;
        let req = test::TestRequest::get().uri("/sse").to_request();

        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = resp.into_body();
        let mut body = pin!(body);

        // first chunk
        let bytes = future::poll_fn(|cx| body.as_mut().poll_next(cx)).await;
        println!("byte 1: {bytes:#?}");

        assert_eq!(
            bytes.unwrap().unwrap(),
            web::Bytes::from_static(b"data: 5\n\n")
        );

        // Second chunk
        let bytes = future::poll_fn(|cx| body.as_pin_mut().poll_next(cx)).await;
        println!("byte 2: {bytes:#?}");
        assert_eq!(
            bytes.unwrap().unwrap(),
            web::Bytes::from_static(b"data: 4\n\n")
        );

        // Remaining part
        for i in 0..3 {
            let expected_data = format!("data: {}\n\n", 3 - i);
            let bytes = future::poll_fn(|cx| body.as_pin_mut().poll_next(cx)).await;
            println!("rem bytes: {bytes:#?}");
            assert_eq!(bytes.unwrap().unwrap(), web::Bytes::from(expected_data));
        }
    }

    #[actix_web::test]
    async fn test_stream_full_payload() {
        let app = test::init_service(App::new().route("/sse", web::get().to(sse))).await;
        let req = test::TestRequest::get().uri("/sse").to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body = resp.into_body();
        let bytes = body::to_bytes(body).await;
        assert_eq!(
            bytes.unwrap(),
            web::Bytes::from_static(b"data: 5\n\ndata: 4\n\ndata: 3\n\ndata: 2\n\ndata: 1\n\n")
        );
    }
}
