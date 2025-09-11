mod cli;
mod error;
mod qck;

use error::Result;
use std::process;

fn main() {
    // Run the application and handle any errors
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}

fn run() -> Result<()> {
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
