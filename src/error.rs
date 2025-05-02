//! Error handling for the QCK Prism XL driver

use thiserror::Error;

/// Custom error types for the QCK Prism XL driver
#[derive(Error, Debug)]
pub enum QckError {
    /// Error when interacting with the USB device
    #[error("USB error: {0}")]
    UsbError(#[from] rusb::Error),

    /// Error when the device cannot be found on the USB bus
    #[error("Device not found: VID={0:#06x}, PID={1:#06x}")]
    DeviceNotFound(u16, u16),

    /// Error when no configurable endpoints are found on the device
    #[error("No configurable endpoints found on device")]
    NoEndpointsFound,

    /// Error when an invalid color format is provided
    #[error("Invalid color format: {0}. Must be a 6-digit hex value (e.g., FF00FF)")]
    InvalidColorFormat(String),

    /// Error when decoding a hex color string
    #[error("Failed to decode hex color: {0}")]
    HexDecodeError(#[from] hex::FromHexError),
}

/// A Result type alias that uses our custom QckError
pub type Result<T> = std::result::Result<T, QckError>;
