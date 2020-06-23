#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to open file: {0}")]
    UnableToOpenFile(#[from] std::io::Error),

    #[error("Unable to parse data: {0}")]
    UnableToParse(#[from] serde_json::Error),

    #[error("Unable to load texture: {0}")]
    UnableToLoadTexture(#[from] image::ImageError),

    #[error("Unable to convert to RGBA8")]
    BadImageColor,
}
