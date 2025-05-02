mod cli;
mod qck;
mod error;

use error::Result;
use clap::Parser;

fn main() -> Result<()> {
    // Parse command line arguments
    let args = cli::CliArgs::parse();
    
    // Send command to the device
    qck::send_to_device(qck::Command {
        light_level: args.light,
        first_color: args.color1,
        second_color: args.color2,
    })?;
    
    println!("Successfully sent command to QCK Prism XL device");
    Ok(())
}
