use clap::{Parser, builder::TypedValueParser};
use crate::qck;
use crate::error::{QckError, Result};

/// A parser for hex color strings (e.g., "FF00FF")
#[derive(Clone, Debug)]
struct HexColorParser;

impl TypedValueParser for HexColorParser {
    type Value = qck::Color;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> std::result::Result<Self::Value, clap::Error> {
        let value_str = value.to_str().ok_or_else(|| {
            let mut err = clap::Error::new(clap::error::ErrorKind::InvalidUtf8);
            err.insert(
                clap::error::ContextKind::InvalidArg,
                clap::error::ContextValue::String(arg.map_or_else(
                    || "color".to_string(),
                    |a| a.get_id().to_string(),
                )),
            );
            err
        })?;

        parse_hex_color(value_str)
            .map_err(|e| {
                let mut err = clap::Error::new(clap::error::ErrorKind::InvalidValue);
                err.insert(
                    clap::error::ContextKind::InvalidArg,
                    clap::error::ContextValue::String(arg.map_or_else(
                        || "color".to_string(),
                        |a| a.get_id().to_string(),
                    )),
                );
                err.insert(
                    clap::error::ContextKind::InvalidValue,
                    clap::error::ContextValue::String(value_str.to_string()),
                );
                err.insert(
                    clap::error::ContextKind::Custom,
                    clap::error::ContextValue::String(e.to_string()),
                );
                err
            })
    }
}

/// Command line arguments for the QCK Prism XL RGB driver
#[derive(Parser)]
#[command(name = "SteelSeries QCK Prism XL RGB driver")]
#[command(author = "Jakub Maciej <zapp88@gmail.com>")]
#[command(version = "0.2.0")]
#[command(about = "This utility allows you to control RGB lighting on your QCK Prism XL")]
pub struct CliArgs {
    /// Sets light level (0-255)
    #[arg(short = 'l', long = "light", default_value = "255", value_parser = clap::value_parser!(u8).range(0..=255))]
    light: u8,

    /// Sets LED1 color in hex (eg. FF00FF)
    #[arg(short = 'a', long = "color1", required = true, value_parser = HexColorParser)]
    color1: qck::Color,

    /// Sets LED2 color in hex (eg. FF00FF)
    #[arg(short = 'b', long = "color2", required = true, value_parser = HexColorParser)]
    color2: qck::Color,
}

/// Processed and validated command line arguments
pub struct Args {
    pub first_color: qck::Color,
    pub second_color: qck::Color,
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

/// Parse a hex color string (e.g., "FF00FF") into an RGB Color
fn parse_hex_color(color_str: &str) -> Result<qck::Color> {
    let decoded = hex::decode(color_str)?;

    match decoded.as_slice() {
        [r, g, b] => Ok(qck::Color { r: *r, g: *g, b: *b }),
        _ => Err(QckError::InvalidColorFormat(color_str.to_string())),
    }
}

/// Parse command line arguments and return validated Args
pub fn fetch_cli_args() -> Result<Args> {
    let cli_args = CliArgs::parse();
    Ok(Args::from(cli_args))
}
