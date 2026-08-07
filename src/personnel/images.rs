use std::fs;

use actix_multipart::form::{MultipartForm, tempfile::TempFile};

#[derive(Debug, MultipartForm)]
pub struct ImageUpload {
    #[multipart(rename = "post_image")]
    pub image: Option<TempFile>,
}

pub fn process_image(image: fs::File) -> anyhow::Result<bool> {
    tracing::info!("Image received: {image:#?}");

    // Parse the file and determine if it is a valid image format (e.g., JPEG, PNG)
    Ok(false)
}
