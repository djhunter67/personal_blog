use actix_web::{HttpResponse, get};
use askama::Template;
use tracing::instrument;

use crate::endpoints::templates::IndexTemplate;

#[get("/logout")]
#[instrument(
    name = "User login attempted",
    level = "info",
    target = "sundayLifeServices web app"
)]
pub async fn logout() -> HttpResponse {
    tracing::debug!("Logout endpoint called");

    // Remove the session_id and clear the cookie

    let version: &str = env!("CARGO_PKG_VERSION");

    let var_name = IndexTemplate {
        title: "Home",
        content: [".lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
", ".lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
"].to_vec(),
        version,
	user: "None"
    };

    let template = var_name.render().expect("Login page render error");

    HttpResponse::Ok().body(template)
}
