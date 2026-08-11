mod png;

use std::fmt;
use std::fs;

use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use tracing::instrument;

#[derive(Debug, MultipartForm)]
pub struct ImageUpload {
    #[multipart(rename = "post_image")]
    pub image: Option<TempFile>,
}
