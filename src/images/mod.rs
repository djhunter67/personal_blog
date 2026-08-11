use actix_multipart::form::{MultipartForm, tempfile::TempFile};

#[derive(Debug, MultipartForm)]
pub struct ImageUpload {
    #[multipart(rename = "post_image")]
    pub image: Option<TempFile>,
}
