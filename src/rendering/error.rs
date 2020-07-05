#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No suitable adapter found")]
    NoSuitableAdapter,

    #[error("No suitable device found")]
    NoSuitableDevice,
}
