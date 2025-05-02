mod cli;
mod qck;
mod error;

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
    let args = cli::fetch_cli_args()?;
    
    // Create and send command to the device
    let command = qck::Command::new(
        args.light_level,
        args.first_color,
        args.second_color,
    );
    qck::send_to_device(command)?;
    
    println!("Successfully sent command to QCK Prism XL device");
    Ok(())
}
