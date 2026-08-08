use std::fs;

use actix_multipart::form::{MultipartForm, tempfile::TempFile};
use futures::join;
use tracing::instrument;

#[derive(Debug, MultipartForm)]
pub struct ImageUpload {
    #[multipart(rename = "post_image")]
    pub image: Option<TempFile>,
}

pub enum ImageType {
    Bmp,
    Gif,
    Jpeg,
    Png,
    Tiff,
    Webp,
}

impl ImageType {
    /// Returns `true` if the image type is [`WEBP`].
    ///
    /// [`WEBP`]: ImageType::WEBP
    #[must_use]
    fn is_webp(&self) -> bool {
        matches!(self, Self::Webp)
    }

    /// Returns `true` if the image type is [`TIFF`].
    ///
    /// [`TIFF`]: ImageType::TIFF
    #[must_use]
    fn is_tiff(&self) -> bool {
        matches!(self, Self::Tiff)
    }

    /// Returns `true` if the image type is [`BMP`].
    ///
    /// [`BMP`]: ImageType::BMP
    #[must_use]
    fn is_bmp(&self) -> bool {
        matches!(self, Self::Bmp)
    }

    /// Returns `true` if the image type is [`GIF`].
    ///
    /// [`GIF`]: ImageType::GIF
    #[must_use]
    fn is_gif(&self) -> bool {
        matches!(self, Self::Gif)
    }

    /// Returns `true` if the image type is [`PNG`].
    ///
    /// [`PNG`]: ImageType::PNG
    #[must_use]
    fn is_png(&self) -> bool {
        matches!(self, Self::Png)
    }

    /// Returns `true` if the image type is [`JPEG`].
    ///
    /// [`JPEG`]: ImageType::JPEG
    #[must_use]
    fn is_jpeg(&self) -> bool {
        matches!(self, Self::Jpeg)
    }
}

/// # Errors
///
///   - Error if the async operation fails for it is computationally extensive
#[instrument(
    name = "Process an image to validate it is an image",
    level = "info",
    target = "Process image",
    skip(image)
)]
pub async fn process_image(image: fs::File) -> anyhow::Result<bool> {
    tracing::info!("Image received: {image:#?}");

    let img_validation = actix_web::web::block(move || validate_img_type(image))
        .await
        .map_err(|err| {
            tracing::error!("Error validating image type: {err:?}",);
            anyhow::anyhow!("Error validating image type: {err:?}")
        })?;

    join!(img_validation).0?;

    // Parse the file and determine if it is a valid image format (e.g., JPEG, PNG)
    Ok(false)
}

async fn validate_img_type(image: fs::File) -> anyhow::Result<ImageType> {
    // Implement your image validation logic here
    // For example, you can check the file extension or use an image processing library to validate the format

    Ok(ImageType::Jpeg) // Placeholder return value
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
