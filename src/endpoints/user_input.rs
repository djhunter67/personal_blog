use std::fmt::Display;

use actix_web::{
    HttpRequest, HttpResponse, delete, post,
    web::{self, Data, Form},
};
use askama::Template;
use mongodb::{
    bson::{DateTime as BsonDateTime, doc},
    options::{FindOneAndUpdateOptions, ReturnDocument},
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    endpoints::templates::{JournalPostEdit, JournalPostEditor},
    models::{
        mongo::{self, JournalDraft},
        redis_conf::{self, authenticated_user_id},
    },
    settings,
};

/// # TODO
///
/// Editing of posts
/// Automatic draft saving
/// Autosave interval
/// Categories of posts
/// Show word count
/// Journaling landing page
/// Sort order of posts
/// Entries per page and pagination
/// Confirmation before deletion && deletion
/// Trash retention period
/// Time zone metadata per post
/// Export journal entries to Markdown or JSON
/// Download all images and journal entries
/// Restore recently deleted posts
/// View the data the application stores
#[derive(Debug, Serialize, Deserialize)]
pub struct BlogPost {
    title: String,
    body: String,
    author: String,
    user_id: String,
    date: BsonDateTime,
    #[serde(default)]
    logged_in: bool,
}

impl BlogPost {
    #[must_use]
    pub fn to_name() -> String {
        String::from("BlogPosts")
    }
    #[must_use]
    pub fn new(
        title: String,
        body: String,
        author: String,
        user_id: String,
        logged_in: bool,
    ) -> Self {
        Self {
            title,
            body,
            author,
            user_id,
            date: BsonDateTime::from_system_time(chrono::Utc::now().into()),
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
    pub const fn get_date(&self) -> &BsonDateTime {
        &self.date
    }

    #[must_use]
    pub fn get_user_id(&self) -> &str {
        &self.user_id
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
            user_id: String::new(),
            logged_in: false,
            date: BsonDateTime::from_system_time(chrono::Utc::now().into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct JournalDraftInput {
    draft_id: Option<String>,
    title: String,
    body: String,
    author: String,
}

impl JournalDraftInput {
    #[must_use]
    pub const fn new(
        draft_id: Option<String>,
        title: String,
        body: String,
        author: String,
    ) -> Self {
        Self {
            draft_id,
            title,
            body,
            author,
        }
    }

    #[must_use]
    pub const fn get_draft_id(&self) -> Option<&String> {
        self.draft_id.as_ref()
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

    pub fn set_draft_id(&mut self, draft_id: Option<String>) {
        self.draft_id = draft_id;
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

    pub fn set_author(&mut self, author: String) {
        self.author = author;
    }

    pub fn clear(&mut self) {
        self.draft_id = None;
        self.title.clear();
        self.body.clear();
        self.author.clear();
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.title.is_empty() && self.body.is_empty() && self.author.is_empty()
    }
}

impl Display for JournalDraftInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Draft ID: {:?}, Title: {}, Body: {}, Author: {}",
            self.draft_id, self.title, self.body, self.author
        )
    }
}

#[allow(clippy::future_not_send)]
#[instrument(
    name = "User submits text",
    level = "info",
    target = "Personal journal",
    skip(req, mongo_client, redis_client, input)
)]
#[post("/submit_text")]
pub async fn submit_text(
    mongo_client: Data<mongodb::Client>,
    redis_client: Data<r2d2::Pool<redis::Client>>,
    Form(input): web::Form<JournalDraftInput>,
    req: HttpRequest,
) -> HttpResponse {
    tracing::info!("Submit text endpoint");

    let user_oid = match authenticated_user_id(&req, &mongo_client, &redis_client).await {
        Ok(user_oid) => user_oid,
        Err(err) => {
            tracing::error!(?err, "Unable to authenticate the user");
            return HttpResponse::Unauthorized().finish();
        }
    };

    let title = input.title.trim();
    let body = input.body.trim();
    let author = input.author.trim();

    if title.is_empty() | body.is_empty() {
        return HttpResponse::BadRequest().body("A title and a body is required");
    }

    let journal_entries = match mongo::establish_connection(&mongo_client).await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!(?err, "Unable to procure a db connection");
            return HttpResponse::InternalServerError().finish();
        }
    }
    .collection::<BlogPost>("BlogPosts");

    // let drafts = match mongo::establish_connection(&mongo_client).await {
    //     Ok(conn) => conn,
    //     Err(err) => {
    //         tracing::error!(?err, "Unable to procure the db connection");
    //         return HttpResponse::InternalServerError().finish();
    //     }
    // }
    // .collection::<JournalDraft>("journal_entries");

    let new_entry = BlogPost::new(
        title.to_owned(),
        body.to_owned(),
        author.to_owned(),
        user_oid.to_string(),
        true,
    );
    tracing::warn!("Created BlogPost instance: {new_entry:#?}");

    match journal_entries.insert_one(&new_entry).await {
        Ok(_) => {
            // tracing::info!("Inserting a new journal entry");
            // if let Err(err) = drafts
            //     .delete_one(doc! {
            //     "_id": user_oid
            //     })
            //     .await
            // {
            //     tracing::error!(
            //         ?err,
            //         %user_oid,
            //         "Entry published but draft cleanup failed"
            //     );
            // }
            let blog_template = JournalPostEdit::new(new_entry);

            let render = match blog_template.render() {
                Ok(html) => html,
                Err(err) => {
                    tracing::error!(
                    ?err,
                    %user_oid,
                    "Entry published but rendering failed"
                    );
                    return HttpResponse::InternalServerError().body(
                        "Entry published but rendering failed. Your saved draft remains available.",
                    );
                }
            };

            HttpResponse::Ok().body(render)
        }
        Err(err) => {
            tracing::error!(
            ?err,
            %user_oid,
            "Unable to publish journal entry"
            );

            HttpResponse::InternalServerError()
                .body("Publishing failed. Yor saved draft remains available.")
        }
    }
}

/// Endpoint to edit a submission. This endpoint retrieves the latest journal entry for the authenticated user and renders it in an editable form.
/// # Errors
///
/// - Returns `HttpResponse::Unauthorized` if the user is not authenticated.
/// - Returns `HttpResponse::InternalServerError` if there is an issue connecting to the database or retrieving the journal entry.
/// - Returns `HttpResponse::NotFound` if no journal entry is found for the authenticated user.
#[allow(clippy::future_not_send)]
#[post("/edit_submission")]
pub async fn edit_submission(
    reids_client: Data<r2d2::Pool<redis::Client>>,
    mongo_client: Data<mongodb::Client>,
    Form(input): web::Form<JournalDraftInput>,
    req: HttpRequest,
) -> HttpResponse {
    tracing::info!("Edit submission endpoint");

    let user_oid = match authenticated_user_id(&req, &mongo_client, &reids_client).await {
        Ok(user_oid) => user_oid,
        Err(err) => {
            tracing::error!(?err, "Unable to authenticate the user");
            return HttpResponse::Unauthorized().finish();
        }
    };

    let entry = BlogPost::new(
        input.title.to_string(),
        input.body.to_string(),
        input.author.to_string(),
        user_oid.to_string(),
        true,
    );

    let edit_template = JournalPostEditor::new(entry);

    let render = match edit_template.render() {
        Ok(html) => html,
        Err(err) => {
            tracing::error!(
                ?err,
                %user_oid,
                "Entry retrieved but rendering failed"
            );
            return HttpResponse::InternalServerError()
                .body("Entry retrieved but rendering failed.");
        }
    };

    HttpResponse::Ok().body(render)
}

/// Update the text of the most recently posted journal entry for the authenticated user.
#[allow(clippy::future_not_send)]
#[post("/update_text")]
pub async fn update_text(
    req: HttpRequest,
    mongo_client: Data<mongodb::Client>,
    redis_client: Data<r2d2::Pool<redis::Client>>,
    Form(input): web::Form<JournalDraftInput>,
) -> HttpResponse {
    tracing::info!("Update text endpoint");

    let user_oid = match authenticated_user_id(&req, &mongo_client, &redis_client).await {
        Ok(user_oid) => user_oid,
        Err(err) => {
            tracing::error!(?err, "Unable to authenticate the user");
            return HttpResponse::Unauthorized().finish();
        }
    };

    let journal_entries = match mongo::establish_connection(&mongo_client).await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!(?err, "Unable to procure a db connection");
            return HttpResponse::InternalServerError().finish();
        }
    }
    .collection::<BlogPost>("BlogPosts");

    // Find the latest journal entry for the authenticated user
    let filter = doc! {
        "user_id": user_oid.to_string(),
    };
    // Get the latest entry by sorting in descending order based on the creation timestamp

    // The exact data to be updated
    let update_doc = doc! {
    "$set": doc! {
        "body": input.get_body(),
        "date": BsonDateTime::now(),
    }
    };

    let options = FindOneAndUpdateOptions::builder()
        .sort(doc! { "date": -1 }) // Sort by date in descending order
        .return_document(ReturnDocument::After) // Return the updated document
        .build();

    let entry: BlogPost = match journal_entries
        .find_one_and_update(filter, update_doc)
        .with_options(options)
        .await
    {
        Ok(Some(entry)) => {
            tracing::warn!("The results of the upadte: {entry:#?}");
            // Some(entry);
            entry
        }
        Ok(None) => {
            tracing::warn!(%user_oid, "No journal entry found for the user to update the posts");
            return HttpResponse::NotFound()
                .body("No journal entry found for the user to update the post");
        }
        Err(err) => {
            tracing::error!(?err, %user_oid, "Unable to retrieve the journal entry");
            return HttpResponse::InternalServerError()
                .body("Unable to retrieve the journal entry");
        }
    };

    // if entry.matched_count.ne(&0) {
    //     tracing::warn!(%user_oid, "User update returned more than one entry: {entry:#?}");
    //     return HttpResponse::InternalServerError()
    //         .body("User update returned more than one entry. This should not happen.");
    // }

    // get the updated entry from the database to render it
    // let updated_entry: BlogPost = match journal_entries
    //     .find_one(doc! { "_id": &user_oid.to_string() })
    //     .await
    // {
    //     Ok(Some(entry)) => entry,
    //     Ok(None) => {
    //         tracing::warn!(
    // 		%user_oid, "No journal entry found for the user after update when querying the updated entry");
    //         return HttpResponse::NotFound()
    //             .body("No journal entry found for the user after update");
    //     }

    //     Err(err) => {
    //         tracing::error!(?err, %user_oid, "Unable to retrieve the updated journal entry");
    //         return HttpResponse::InternalServerError()
    //             .body("Unable to retrieve the updated journal entry");
    //     }
    // };

    let edit_template = JournalPostEditor::new(entry);

    let render = match edit_template.render() {
        Ok(html) => html,
        Err(err) => {
            tracing::error!(
                ?err,
                %user_oid,
                "Entry retrieved but rendering failed"
            );
            return HttpResponse::InternalServerError()
                .body("Entry retrieved but rendering failed.");
        }
    };

    HttpResponse::Ok().body(render)
}

/// Delete the most immediately posted post from the user
#[allow(clippy::future_not_send)]
#[delete("/delete_submission")]
pub async fn delete_submission(
    req: HttpRequest,
    mongo_client: Data<mongodb::Client>,
    redis_client: Data<r2d2::Pool<redis::Client>>,
) -> HttpResponse {
    tracing::info!("Delete submission endpoint");

    let user_oid = match authenticated_user_id(&req, &mongo_client, &redis_client).await {
        Ok(user_oid) => user_oid,
        Err(err) => {
            tracing::error!(?err, "Unable to authenticate the user");
            return HttpResponse::Unauthorized().finish();
        }
    };

    let journal_entries = match mongo::establish_connection(&mongo_client).await {
        Ok(conn) => conn,
        Err(err) => {
            tracing::error!(?err, "Unable to procure a db connection");
            return HttpResponse::InternalServerError().finish();
        }
    }
    .collection::<BlogPost>("BlogPosts");

    let filter = doc! {
        "$query": {
            "user_id": user_oid.to_string()
    },
        "$orderby": {
            "date": -1
        },
        "$limit": 1
    };

    match journal_entries.delete_one(filter).await {
        Ok(deleted_entry) => {
            tracing::info!(%user_oid, "Deleted journal entry: {deleted_entry:#?}");
            HttpResponse::Ok().json("Journal entry deleted successfully")
        }
        Err(err) => {
            tracing::error!(?err, %user_oid, "Unable to delete the journal entry");
            HttpResponse::InternalServerError().json("Unable to delete the journal entry")
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

    let mut red_conn = match redis_conf::establish_connection(redis) {
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

    tracing::warn!("The session key: {session_key}");

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

// #[allow(clippy::future_not_send)]
// #[post("/draft/autosave")]
// pub async fn autosave_journal_draft(
//     req: HttpRequest,
//     mongo_client: Data<mongodb::Client>,
//     redis_client: Data<r2d2::Pool<redis::Client>>,
//     Form(input): Form<JournalDraftInput>,
// ) -> HttpResponse {
//     let user_id = match authenticated_user_id(&req, &mongo_client, &redis_client).await {
//         Ok(user_id) => user_id,
//         Err(AuthenticationError::MissingSession | AuthenticationError::InvalidSession) => {
//             return HttpResponse::Unauthorized()
//                 .body("Your session expired. The draft was not saved");
//         }
//         Err(AuthenticationError::Redis) => {
//             tracing::error!("Unable to access Redis during draft autosave");

//             return HttpResponse::ServiceUnavailable()
//                 .body("Draft autosave is temporarily unavailable");
//         }
//     };

//     let title = input.title.trim();
//     let author = input.author.trim();

//     if title.chars().count() > 300 {
//         return HttpResponse::BadRequest().body("Draft was not saved: title is too long");
//     }
//     if input.body.chars().count() > 1_000_000 {
//         return HttpResponse::BadRequest().body("Draft was not saved: journal entry is too long");
//     }

//     if author.chars().count() > 200 {
//         return HttpResponse::BadRequest().body("Draft was not saved: author name is too long");
//     }

//     // Check if the meaningful fields have been filled in
//     if title.is_empty() && input.body.trim().is_empty() {
//         return HttpResponse::Ok().body("Begin typing to create a draft");
//     }

//     let drafts = mongo::establish_connection(&mongo_client)
//         .await
//         .expect("mongo error")
//         .collection::<JournalDraft>("journal_drafts");

//     let now = bson::DateTime::now();

//     let draft_id =
//         match bson::oid::ObjectId::parse_str(input.draft_id.as_ref().unwrap_or(&"".to_string())) {
//             Ok(draft_id) => draft_id,
//             Err(err) => {
//                 tracing::error!(
//                     ?err,
//                     "Unable to parse the draft_id: {}",
//                     input.draft_id.as_ref().unwrap_or(&"".to_string())
//                 );
//                 return HttpResponse::BadRequest().body("Draft was not saved: invalid draft_id");
//             }
//         };

//     let filter = doc! {
//     "_id": draft_id,
//     "user_id": user_id,
//     "state": "active"
//     };

//     let update = doc! {
//     "$set": {
//         "title": title,
//         "body": &input.body,
//         "author": author,
//         "updated_at": now,
//     },
//     "$setOnInsert": {
//         "_id": mongodb::bson::oid::ObjectId::new(),
//         "user_id": user_id,
//         "created_at": now,
//     },
//     "$inc": {
//         "revision": 1_i64
//     },
//     };

//     let update_result = drafts.update_one(filter, update).upsert(true).await;

//     match update_result {
//         Ok(_) => {
//             tracing::warn!("DB update successful for user_id: {user_id}");
//             let displayed_time = chrono::Utc::now().format("%Y-%m-%d %H:%S UTC");

//             let message = format!("Draft saved at {displayed_time}.");

//             let response = DraftStatusTemplate {
//                 status_class: "saved",
//                 message: &message,
//             };

//             match response.render() {
//                 Ok(rend) => HttpResponse::Ok()
//                     .content_type("text/html; charset=utf-8")
//                     .body(rend),

//                 Err(err) => {
//                     tracing::error!("Unable to render draft status: {err}");
//                     HttpResponse::InternalServerError().finish()
//                 }
//             }
//         }
//         Err(err) => {
//             tracing::error!(
//             ?err,
//             %user_id,
//             "Unable to autosave journal draft"
//             );

//             HttpResponse::InternalServerError()
//                 .body("The draft could not be saved. Continue typing and try again")
//         }
//     }
// }

/// # Errors
///
/// If the database connection fails, or if the query fails, this function will return a `mongodb::error::Error`.
pub async fn find_active_draft(
    mongo: &mongodb::Client,
    user_id: mongodb::bson::oid::ObjectId,
) -> mongodb::error::Result<Option<JournalDraft>> {
    let drafts = mongo::establish_connection(mongo)
        .await?
        .collection::<JournalDraft>("journal_drafts");

    drafts
        .find_one(doc! {
        "user_id": user_id
        })
        .await
}

// #[allow(clippy::future_not_send)]
// #[delete("draft/current")]
// pub async fn discard_current_draft(
//     req: HttpRequest,
//     mongo_client: Data<mongodb::Client>,
//     redis_client: Data<r2d2::Pool<redis::Client>>,
// ) -> HttpResponse {
//     let user_id = match authenticated_user_id(&req, &mongo_client, &redis_client).await {
//         Ok(user_id) => user_id,
//         Err(AuthenticationError::MissingSession | AuthenticationError::InvalidSession) => {
//             return HttpResponse::Unauthorized().body("Your session has expired");
//         }
//         Err(AuthenticationError::Redis) => {
//             tracing::error!("The cache-layer could not be established");
//             return HttpResponse::ServiceUnavailable()
//                 .json("The draft could not be discarded".to_string());
//         }
//     };

//     let drafts = match mongo::establish_connection(&mongo_client).await {
//         Ok(db) => db,
//         Err(err) => {
//             tracing::error!("Unable to procure the database: {err}");
//             return HttpResponse::InternalServerError().body("Failed to procure the db: {err}");
//         }
//     }
//     .collection::<JournalDraft>("journal_drafts");

//     match drafts
//         .delete_one(doc! {
//             "user_id": user_id,
//         })
//         .await
//     {
//         Ok(_) => {
//             tracing::info!("Successfully deleted an entry");
//             let empty_form = JournalFormTemplate::new(vec![BlogPost::default()]);
//             match empty_form.render() {
//                 Ok(html) => HttpResponse::Ok()
//                     .content_type("text/html; charset=utf-8")
//                     .body(html),
//                 Err(err) => {
//                     tracing::error!(?err, %user_id, "Unable to save journal draft");
//                     HttpResponse::InternalServerError().finish()
//                 }
//             }
//         }
//         Err(err) => {
//             tracing::error!(?err, "Unable to render empty journal form");
//             HttpResponse::InternalServerError().body("The draft could not be discarded")
//         }
//     }
// }
