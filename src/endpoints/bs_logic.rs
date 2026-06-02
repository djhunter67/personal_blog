use actix_web::{HttpResponse, get, http::header::ContentType};
use askama::Template;

#[derive(Template)]
#[template(path = "about.html")]
struct AboutTemplate<'a> {
    title: &'a str,
    content: Vec<&'a str>,
}

#[derive(Template)]
#[template(path = "schedule.html")]
struct ScheduleTemplate<'a> {
    title: &'a str,
    content: Vec<&'a str>,
}

#[derive(Template)]
#[template(path = "testimonials.html")]
struct TestimonialTemplate<'a> {
    title: &'a str,
    content: Vec<&'a str>,
}

#[derive(Template)]
#[template(path = "finances.html")]
struct FinancesTemplate<'a> {
    title: &'a str,
    content: Vec<&'a str>,
}

#[derive(Template)]
#[template(path = "contact.html")]
struct ContactTemplate<'a> {
    title: &'a str,
    content: Vec<&'a str>,
}

#[get("/about")]
pub async fn about() -> HttpResponse {
    let company_origins: &str = "The company started in Golden Valley, Arizona in 2006";
    let owner_info: &str = "Nahan Loka is the sole proprietor of SundayLife Services";
    let template = AboutTemplate {
        title: "About",
        content: [company_origins, owner_info].to_vec(),
    };

    let template = template.render().expect("About page render error");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(template)
}

#[get("/schedule")]
pub async fn schedule() -> HttpResponse {
    let open_dates: &str = "All the open and available dates";
    let closed_dates: &str = "These dates have been reserved";
    let canceled: &str = "Cancellations";
    let template = ScheduleTemplate {
        title: "Schedule",
        content: [open_dates, closed_dates, canceled].to_vec(),
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
        title: "Testimonials",
        content: [customer_feedback, ratings, dates_of_service].to_vec(),
    };

    let template = template.render().expect("About page render error");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(template)
}

#[get("/finances")]
pub async fn finances() -> HttpResponse {
    let finances_benefit: &str = "The company started is great!";
    let financial_aid: &str = "Five Stars";
    let customer_value: &str = "A DateTime object";
    let template = FinancesTemplate {
        title: "Costs",
        content: [finances_benefit, financial_aid, customer_value].to_vec(),
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
        title: "Contact",
        content: [business_contact, personal_contact, business_email].to_vec(),
    };

    let template = template.render().expect("About page render error");

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(template)
}
