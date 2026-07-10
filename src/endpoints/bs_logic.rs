/// Business logic lives here
use actix_web::{HttpRequest, HttpResponse, get, http::header::ContentType, web::Data};
use askama::Template;

use crate::{
    models::redis_conf::{self},
    settings,
};

#[derive(Template)]
#[template(path = "parts/about.part.html")]
struct AboutTemplate<'a> {
    content: Vec<&'a str>,
    user: &'a str,
}

#[derive(Template)]
#[template(path = "parts/schedule.part.html")]
struct ScheduleTemplate<'a> {
    content: Vec<&'a str>,
    user: &'a str,
}

#[derive(Template)]
#[template(path = "parts/testimonials.parts.html")]
struct TestimonialTemplate<'a> {
    content: Vec<&'a str>,
    user: &'a str,
}

#[derive(Template)]
#[template(path = "parts/finances.part.html")]
struct FinancesTemplate<'a> {
    content: Vec<&'a str>,
    user: &'a str,
}

#[derive(Template)]
#[template(path = "parts/contact.parts.html")]
struct ContactTemplate<'a> {
    content: Vec<&'a str>,
    user: &'a str,
}

#[allow(clippy::future_not_send)]
#[get("/about")]
pub async fn about(req: HttpRequest, redis: Data<r2d2::Pool<redis::Client>>) -> HttpResponse {
    tracing::info!("About page loading");
    let session_id = if let Some(cookie) = req.cookie("session_id") {
        cookie.value().to_string()
    } else {
        tracing::error!("User cookie not found: {req:#?}");
        return HttpResponse::Unauthorized().body(format!(
            "No session found: {:#?}",
            req.cookies().expect("No cookies found")
        ));
    };

    let mut red_conn = match redis_conf::establish_connection(redis.get_ref().clone()) {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!("Unable to acquire the cache layer connection: {err:#?}");
            return HttpResponse::InternalServerError()
                .body(format!("Cache layer error: {err:#?}"));
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
            return HttpResponse::InternalServerError()
                .body(format!("Unable to acquire the cache layer: {err:#?}"));
        }
    };

    tracing::debug!("The session id: {session_key}");
    user.map_or_else(
        || {
            tracing::error!("No user data associated with the recieved session key: {session_key}");
            HttpResponse::InternalServerError().body("No user data found")
        },
        |email| {
            let company_origins: &str = "The company started in Golden Valley, Arizona in 2006";
            let owner_info: &str = "Nahan Loka is the sole proprietor of SundayLife Services";
            let template = AboutTemplate {
                content: [company_origins, owner_info].to_vec(),
                user: &email,
            };

            let template = template.render().expect("About page render error");

            HttpResponse::Ok()
                .content_type(ContentType::html())
                .body(template)
        },
    )
}

#[get("/schedule")]
pub async fn schedule() -> HttpResponse {
    let open_dates: &str = "All the open and available dates";
    let closed_dates: &str = "These dates have been reserved";
    let canceled: &str = "Cancellations";
    let template = ScheduleTemplate {
        content: [open_dates, closed_dates, canceled].to_vec(),
        user: "logged in user",
    };

    let template = template.render().expect("About page render error");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(template)
}

#[get("/testimonials")]
pub async fn testimonials() -> HttpResponse {
    let customer_feedback: &str = "The company started is great!";
    let ratings: &str = "Five Stars";
    let dates_of_service: &str = "A DateTime object";
    let template = TestimonialTemplate {
        content: [customer_feedback, ratings, dates_of_service].to_vec(),
        user: "logged in user",
    };

    let template = template.render().expect("About page render error");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(template)
}

#[get("/finances")]
pub async fn finances() -> HttpResponse {
    let finances_benefit: &str = "Hiring quality employees!";
    let financial_aid: &str = "To be determined";
    let customer_value: &str = "The value provided to a customer from our services";
    let template = FinancesTemplate {
        content: [finances_benefit, financial_aid, customer_value].to_vec(),
        user: "logged in user",
    };

    let template = template.render().expect("About page render error");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(template)
}

#[get("/contact")]
pub async fn contact() -> HttpResponse {
    let business_contact: &str = "(623) 800-2580";
    let personal_contact: &str = "(623) 555-2560";
    let business_email: &str = "nahan@sundaylifeservices.com";
    let template = ContactTemplate {
        content: [business_contact, personal_contact, business_email].to_vec(),
        user: "logged in user",
    };

    let template = template.render().expect("About page render error");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(template)
}
