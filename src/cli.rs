use clap::Parser;

use crate::qck;

/// Command line arguments for the QCK Prism XL RGB driver
#[derive(Parser)]
#[command(name = "SteelSeries QCK Prism XL RGB driver")]
#[command(author = "Jakub Maciej <zapp88@gmail.com>")]
#[command(version = "0.2.0")]
#[command(about = "This utility allows you to control RGB lighting on your QCK Prism XL")]
pub struct CliArgs {
    /// Sets light level (0-255)
    #[arg(short = 'l', long = "light", default_value_t = 255, value_parser = clap::value_parser!(u8).range(0..=255))]
    light: u8,

    /// Sets LED1 color in hex (eg. FF00FF)
    #[arg(short = 'a', long = "color1", required = true)]
    color1: qck::Color,

    /// Sets LED2 color in hex (eg. FF00FF)
    #[arg(short = 'b', long = "color2", required = true)]
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

/// Parse command line arguments and return validated Args
pub fn fetch_cli_args() -> Args {
    CliArgs::parse().into()
}
