use thiserror::Error;

#[derive(Error, Debug)]
pub enum QckError {
    #[error("USB error: {0}")]
    UsbError(#[from] rusb::Error),

    #[error("Device not found: VID={0:04x}, PID={1:04x}")]
    DeviceNotFound(u16, u16),

    #[error("No configurable endpoints found on device")]
    NoEndpointsFound,

    #[error("Invalid light level: {0}. Must be between 0 and 255")]
    InvalidLightLevel(i32),

    #[error("Invalid color format: {0}. Must be a 6-digit hex value (e.g., FF00FF)")]
    InvalidColorFormat(String),

    #[error("Failed to decode hex color: {0}")]
    HexDecodeError(#[from] hex::FromHexError),
}

pub type Result<T> = std::result::Result<T, QckError>;
