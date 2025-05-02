use thiserror::Error;

#[derive(Error, Debug)]
pub enum QckError {
    #[error("USB error: {0}")]
    UsbError(#[from] rusb::Error),

    #[error("Device not found: VID={0:04x}, PID={1:04x}")]
    DeviceNotFound(u16, u16),

    #[error("No configurable endpoints found on device")]
    NoEndpointsFound,

    #[error("Failed to decode hex color: {0}")]
    HexDecodeError(#[from] hex::FromHexError),
}

pub type Result<T> = std::result::Result<T, QckError>;
