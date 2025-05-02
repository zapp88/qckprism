use clap::{error::ErrorKind, Parser};
use crate::qck;

/// Parse a hex color string (e.g., "FF00FF") into an RGB Color
/// Returns a clap::Error on failure for direct use with value_parser.
fn parse_hex_color(color_str: &str) -> std::result::Result<qck::Color, clap::Error> {
    // Validate the color string format (should be 6 hex characters)
    if color_str.len() != 6 || !color_str.chars().all(|c| c.is_ascii_hexdigit()) {
        let err_msg = format!(
            "Invalid color format: '{}'. Must be a 6-digit hex value (e.g., FF00FF)",
            color_str
        );
        return Err(clap::Error::raw(ErrorKind::InvalidValue, err_msg));
    }

    // Decode the hex string
    let decoded = hex::decode(color_str).map_err(|e| {
        let err_msg = format!("Failed to decode hex color '{}': {}", color_str, e);
        clap::Error::raw(ErrorKind::InvalidValue, err_msg)
    })?;

    // Ensure we have exactly 3 bytes (RGB)
    // Note: hex::decode already ensures the output length is len / 2,
    // so combining with the length check above, this check is slightly redundant,
    // but kept for clarity.
    if decoded.len() != 3 {
        // This case should technically be unreachable if the length and decode checks pass
        let err_msg = format!(
            "Invalid decoded length for color: '{}'. Expected 3 bytes, got {}",
            color_str,
            decoded.len()
        );
        return Err(clap::Error::raw(ErrorKind::InvalidValue, err_msg));
    }

    Ok(qck::Color {
        r: decoded[0],
        g: decoded[1],
        b: decoded[2],
    })
}

/// Command line arguments for the QCK Prism XL RGB driver
#[derive(Parser, Debug)]
#[command(name = "SteelSeries QCK Prism XL RGB driver")]
#[command(author = "Jakub Maciej <zapp88@gmail.com>")]
#[command(version = "0.2.0")]
#[command(about = "This utility allows you to control RGB lighting on your QCK Prism XL")]
pub struct CliArgs {
    /// Sets light level (0-255)
    #[arg(short = 'l', long = "light", default_value = "255", value_parser = clap::value_parser!(u8).range(0..=255))]
    pub light_level: u8,

    /// Sets LED1 color in hex (eg. FF00FF)
    #[arg(short = 'a', long = "color1", required = true, value_parser = parse_hex_color)]
    pub first_color: qck::Color,

    /// Sets LED2 color in hex (eg. FF00FF)
    #[arg(short = 'b', long = "color2", required = true, value_parser = parse_hex_color)]
    pub second_color: qck::Color,
}

/// Parse command line arguments. Exits on error.
pub fn fetch_cli_args() -> CliArgs {
    CliArgs::parse()
}
