//! SteelSeries QCK Prism XL RGB driver
//! 
//! This utility allows controlling RGB lighting on a SteelSeries QCK Prism XL mousepad.
//! It uses USB HID commands to configure the colors and brightness of the device.

mod cli;
mod error;
mod qck;

use error::Result;
use std::process;

/// Application entry point
fn main() {
    // Run the application and handle any errors
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}

/// Main application logic
fn run() -> Result<()> {
    // Parse command line arguments
    let args = cli::parse_args()?;
    
    // Convert args to command and send to the device
    let command = qck::Command::from(args);
    qck::send_to_device(command)?;
    
    println!("Successfully configured QCK Prism XL device");
    Ok(())
}
