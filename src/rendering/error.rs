use vulkano::command_buffer::*;
use vulkano::device::DeviceCreationError;
use vulkano::OomError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No device available")]
    NoDeviceAvailable,

    #[error("No suitable queues found for: {}", device)]
    NoSuitableQueuesFound { device: String },

    #[error("Unable to create device: {0}")]
    DeviceCreation(#[from] DeviceCreationError),
}
