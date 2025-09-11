mod cli;
mod error;
mod qck;

use error::Result;

fn main() -> Result<()> {
    // Parse command line arguments
    let args = cli::fetch_cli_args();

    // Send command to the device
    qck::send_to_device(qck::Command {
        light_level: args.light_level,
        first_color: args.first_color,
        second_color: args.second_color,
    })?;

    println!("Successfully sent command to QCK Prism XL device");
    Ok(())
}