use std::fs;

use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use tracing::instrument;

#[derive(Debug, MultipartForm)]
pub struct ImageUpload {
    #[multipart(rename = "post_image")]
    pub image: Option<TempFile>,
}

enum ImageType {
    JPEG,
    PNG,
    GIF,
    BMP,
    TIFF,
    WEBP,
}

#[instrument(
    name = "Process an image to validate it is an image",
    level = "info",
    target = "Process image",
    skip(image)
)]
pub async fn process_image(image: fs::File) -> anyhow::Result<bool> {
    tracing::info!("Image received: {image:#?}");

    actix_web::web::block(move || validate_img_type(image))
        .await
        .map_err(|e| {
            tracing::error!("Error validating image type: {:?}", e);
            anyhow::anyhow!("Error validating image type: {:?}", e)
        })?;

    // Parse the file and determine if it is a valid image format (e.g., JPEG, PNG)
    Ok(false)
}

async fn validate_img_type(image: fs::File) -> anyhow::Result<ImageType> {
    // Implement your image validation logic here
    // For example, you can check the file extension or use an image processing library to validate the format

    Ok(ImageType::JPEG) // Placeholder return value
}

async fn save_image(image: fs::File, path: &str) -> anyhow::Result<()> {
    // Implement your image saving logic here
    // For example, you can use the `std::fs` module to save the image to the specified path
    Ok(())
}

async fn delete_image(path: &str) -> anyhow::Result<()> {
    // Implement your image deletion logic here
    // For example, you can use the `std::fs` module to delete the image at the specified path
    Ok(())
}

async fn validate_is_an_image(image: fs::File) -> anyhow::Result<bool> {
    // Implement your image validation logic here
    // For example, you can use an image processing library to validate the image format
    Ok(true)
}

async fn validate_image_size(image: fs::File, max_size: u64) -> anyhow::Result<bool> {
    // Implement your image size validation logic here
    // For example, you can check the file size and compare it to the max_size parameter
    Ok(true)
}

fn create_new_image(image: fs::File, path: &str) -> anyhow::Result<()> {
    // Implement your image creation logic here
    // For example, you can use the `std::fs` module to create a new image file at the specified path

    // Encode the image to the desired format (e.g., JPEG, PNG)

    Ok(())
}
