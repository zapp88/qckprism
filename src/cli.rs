use clap::Parser;
use crate::qck;
use crate::error::{QckError, Result};

#[derive(Parser)]
#[command(name = "SteelSeries QCK Prism XL RGB driver")]
#[command(author = "Jakub Maciej <zapp88@gmail.com>")]
#[command(version = "0.2.0")]
#[command(about = "This utility allows you to control RGB lighting on your QCK Prism XL")]
pub struct CliArgs {
    /// Sets light level (0-255)
    #[arg(short = 'l', long = "light", default_value = "255")]
    light: String,

    /// Sets LED1 color in hex (eg. FF00FF)
    #[arg(short = 'a', long = "color1", required = true)]
    color1: String,

    /// Sets LED2 color in hex (eg. FF00FF)
    #[arg(short = 'b', long = "color2", required = true)]
    color2: String,
}

pub struct Args {
    pub first_color: qck::Color,
    pub second_color: qck::Color,
    pub light_level: u8,
}

/// Parse a hex color string (e.g., "FF00FF") into RGB components
fn parse_hex_color(color_str: &str) -> Result<(u8, u8, u8)> {
    // Validate the color string format (should be 6 hex characters)
    if color_str.len() != 6 || !color_str.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(QckError::InvalidColorFormat(color_str.to_string()));
    }

    // Decode the hex string
    let decoded = hex::decode(color_str)?;
    
    // Ensure we have exactly 3 bytes (RGB)
    if decoded.len() != 3 {
        return Err(QckError::InvalidColorFormat(color_str.to_string()));
    }

    Ok((decoded[0], decoded[1], decoded[2]))
}

pub fn fetch_cli_args() -> Result<Args> {
    let cli_args = CliArgs::parse();

    // Parse light level
    let light_num = cli_args.light.parse::<i32>()
        .map_err(|_| QckError::InvalidLightLevel(cli_args.light.parse().unwrap_or(-1)))?;
    
    if light_num > 255 || light_num < 0 {
        return Err(QckError::InvalidLightLevel(light_num));
    }

    // Parse colors
    let (r1, g1, b1) = parse_hex_color(&cli_args.color1)?;
    let (r2, g2, b2) = parse_hex_color(&cli_args.color2)?;

    Ok(Args {
        first_color: qck::Color { r: r1, g: g1, b: b1 },
        second_color: qck::Color { r: r2, g: g2, b: b2 },
        light_level: light_num as u8,
    })
}
