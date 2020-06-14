use snafu::*;
use vulkano::device::DeviceCreationError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
    #[snafu(display("No device available"))]
    NoDeviceAvailable,

    #[snafu(display("No suitable queues found for: {}", device))]
    NoSuitableQueuesFound { device: String },

    #[snafu(display("Unable to create device: {}", source))]
    DeviceCreation { source: DeviceCreationError },
}
