#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unsupported color type: {0:?}")]
    UnsupportedColorType(png::ColorType),

    #[error("Unsupported bit depth: {0:?}")]
    UnsupportedBitDepth(png::BitDepth),
}
