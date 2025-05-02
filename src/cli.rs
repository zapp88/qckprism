use crate::error::{QckError, Result};
use crate::qck::{self, Color};
use clap::{builder::TypedValueParser, error::ContextKind, error::ContextValue, Parser};
use std::ffi::OsStr;

/// A parser for hex color strings (e.g., "FF00FF")
#[derive(Clone, Debug)]
struct HexColorParser;

impl TypedValueParser for HexColorParser {
    type Value = Color;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &OsStr,
    ) -> std::result::Result<Self::Value, clap::Error> {
        // Get the argument name for error reporting
        let arg_name = arg
            .map(|a| a.get_id().to_string())
            .unwrap_or_else(|| "color".to_string());
        
        // Convert OsStr to &str, handling any UTF-8 conversion errors
        let value_str = value.to_str().ok_or_else(|| {
            let mut err = clap::Error::new(clap::error::ErrorKind::InvalidUtf8);
            err.insert(ContextKind::InvalidArg, ContextValue::String(arg_name.clone()));
            err
        })?;

        // Parse the hex color string
        parse_hex_color(value_str).map_err(|e| {
            let mut err = clap::Error::new(clap::error::ErrorKind::InvalidValue);
            err.insert(ContextKind::InvalidArg, ContextValue::String(arg_name));
            err.insert(ContextKind::InvalidValue, ContextValue::String(value_str.to_string()));
            err.insert(ContextKind::Custom, ContextValue::String(e.to_string()));
            err
        })
    }
}

/// Command line arguments for the QCK Prism XL RGB driver
#[derive(Parser, Debug)]
#[command(name = "SteelSeries QCK Prism XL RGB driver")]
#[command(author = "Jakub Maciej <zapp88@gmail.com>")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Control RGB lighting on your QCK Prism XL mousepad")]
pub struct CliArgs {
    /// Sets light level (0-255)
    #[arg(
        short = 'l', 
        long = "light", 
        default_value = "255", 
        value_parser = clap::value_parser!(u8).range(0..=255),
        help = "Set the brightness level (0-255, where 255 is maximum brightness)"
    )]
    light: u8,

    /// Sets LED1 color in hex (eg. FF00FF)
    #[arg(
        short = 'a', 
        long = "color1", 
        required = true, 
        value_parser = HexColorParser,
        help = "Set the first LED color as a 6-digit hex code (e.g., FF00FF for purple)"
    )]
    color1: Color,

    /// Sets LED2 color in hex (eg. FF00FF)
    #[arg(
        short = 'b', 
        long = "color2", 
        required = true, 
        value_parser = HexColorParser,
        help = "Set the second LED color as a 6-digit hex code (e.g., 00FF00 for green)"
    )]
    color2: Color,
}

/// Processed and validated command line arguments
#[derive(Debug, Clone)]
pub struct Args {
    pub first_color: Color,
    pub second_color: Color,
    pub light_level: u8,
}

impl From<CliArgs> for Args {
    fn from(args: CliArgs) -> Self {
        Self {
            first_color: args.color1,
            second_color: args.color2,
            light_level: args.light,
        }
    }
}

impl From<Args> for qck::Command {
    fn from(args: Args) -> Self {
        Self {
            light_level: args.light_level,
            first_color: args.first_color,
            second_color: args.second_color,
        }
    }
}

/// Parse a hex color string (e.g., "FF00FF") into an RGB Color
fn parse_hex_color(color_str: &str) -> Result<Color> {
    // Validate the color string format (should be 6 hex characters)
    if color_str.len() != 6 || !color_str.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(QckError::InvalidColorFormat(color_str.to_string()));
    }

    // Decode the hex string
    let decoded = hex::decode(color_str)?;
    
    // Convert to a Color
    Color::from_rgb_bytes(&decoded)
}

/// Parse command line arguments and return validated Args
pub fn parse_args() -> Result<Args> {
    let cli_args = CliArgs::parse();
    Ok(Args::from(cli_args))
}
