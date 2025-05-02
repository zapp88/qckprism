use rusb::{Context, Device, DeviceHandle, UsbContext};
use std::time::Duration;
use crate::error::{QckError, Result};

// QCK Prism XL device identifiers
const VID: u16 = 0x1038; // Vendor ID
const PID: u16 = 0x150d; // Product ID

// USB control transfer constants
const REQUEST_TYPE: u8 = 0x21;
const REQUEST: u8 = 0x09;
const ACK_VALUE: u16 = 0x0200;
const LIGHT_VALUE: u16 = 0x0200;
const COLOR_VALUE: u16 = 0x0300;
const INDEX: u16 = 0x0000;
const TIMEOUT: Duration = Duration::from_secs(1);

// Command byte identifiers
const CMD_ACK: u8 = 0x0d;
const CMD_LIGHT: u8 = 0x0c;
const CMD_COLOR: u8 = 0x0e;
const CMD_COLOR_COUNT: u8 = 0x02;

// Color effect constants
const ALPHA_FULL: u8 = 0xff;
const EFFECT_SPEED: u8 = 0x32; // 50
const EFFECT_INTENSITY: u8 = 0xc8; // 200
const EFFECT_FLAG: u8 = 0x01;

/// Represents a USB endpoint configuration
#[derive(Debug, Clone, PartialEq, Eq)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
}

/// Command to send to the QCK Prism XL device
#[derive(Debug, Clone)]
pub struct Command {
    pub light_level: u8,
    pub first_color: Color,
    pub second_color: Color,
}

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Creates a new RGB color
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl Command {
    /// Creates a new command with the specified light level and colors
    pub fn new(light_level: u8, first_color: Color, second_color: Color) -> Self {
        Self {
            light_level,
            first_color,
            second_color,
        }
    }
}

/// Sends a command to the QCK Prism XL device
pub fn send_to_device(command: Command) -> Result<()> {
    // Initialize USB context
    let context = Context::new().map_err(QckError::UsbError)?;
    
    // Open the device and execute the command
    with_device(context, VID, PID, |handle, _endpoint| {
        // Send commands to the device
        set_light(command.light_level, handle)?;
        set_color(handle, command.first_color, command.second_color)?;
        send_ack(handle)?;
        
        Ok(())
    })
}

/// Executes an operation with a properly configured device handle
fn with_device<T, F>(context: Context, vid: u16, pid: u16, operation: F) -> Result<T>
where
    F: FnOnce(&mut DeviceHandle<Context>, &Endpoint) -> Result<T>,
{
    // Open the device
    let (device, mut handle) = open_device(&context, vid, pid)?;

    // Find and configure endpoints
    let endpoints = find_readable_endpoints(&device)?;
    let endpoint = endpoints.first().ok_or(QckError::NoEndpointsFound)?;

    // Handle kernel driver if active
    let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
        Ok(true) => {
            handle.detach_kernel_driver(endpoint.iface).map_err(QckError::UsbError)?;
            true
        }
        _ => false,
    };

    // Configure the endpoint
    configure_endpoint(&mut handle, endpoint)?;

    // Execute the provided operation
    let result = operation(&mut handle, endpoint);

    // Cleanup: release interface and reattach kernel driver if needed
    handle.release_interface(endpoint.iface).map_err(QckError::UsbError)?;
    if has_kernel_driver {
        handle.attach_kernel_driver(endpoint.iface).map_err(QckError::UsbError)?;
    }
    
    result
}

/// Opens the USB device with the specified vendor and product IDs
fn open_device<T: UsbContext>(
    context: &T,
    vid: u16,
    pid: u16,
) -> Result<(Device<T>, DeviceHandle<T>)> {
    let devices = context.devices().map_err(QckError::UsbError)?;

    devices.iter()
        .filter_map(|device| {
            // Try to get device descriptor, skip if error
            let device_desc = device.device_descriptor().ok()?;
            
            // Check if this is the device we're looking for
            if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
                // Try to open the device, return if successful
                device.open().ok().map(|handle| (device, handle))
            } else {
                None
            }
        })
        .next()
        .ok_or(QckError::DeviceNotFound(vid, pid))
}

/// Finds all readable endpoints on the device
fn find_readable_endpoints<T: UsbContext>(device: &Device<T>) -> Result<Vec<Endpoint>> {
    let device_desc = device.device_descriptor().map_err(QckError::UsbError)?;
    let mut endpoints = Vec::new();
    
    // Iterate through configurations
    for config_idx in 0..device_desc.num_configurations() {
        // Get configuration descriptor, skip if error
        let config_desc = match device.config_descriptor(config_idx) {
            Ok(desc) => desc,
            Err(_) => continue,
        };
        
        let config_number = config_desc.number();
        
        // Iterate through interfaces
        for interface in config_desc.interfaces() {
            // Iterate through interface descriptors
            for interface_desc in interface.descriptors() {
                let iface_number = interface_desc.interface_number();
                let setting_number = interface_desc.setting_number();
                
                // For each endpoint descriptor, create an Endpoint
                for _ in interface_desc.endpoint_descriptors() {
                    endpoints.push(Endpoint {
                        config: config_number,
                        iface: iface_number,
                        setting: setting_number,
                    });
                }
            }
        }
    }

    if endpoints.is_empty() {
        return Err(QckError::NoEndpointsFound);
    }

    Ok(endpoints)
}

/// Configures the USB endpoint
fn configure_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) -> Result<()> {
    // Setting same configuration as active twice results in "device is busy error"
    // So we make sure we don't set it again when it is already active
    let active_config = handle.active_configuration().map_err(QckError::UsbError)?;
    
    // Only set configuration if it's different from the current one
    if active_config != endpoint.config {
        handle.set_active_configuration(endpoint.config).map_err(QckError::UsbError)?;
    }
    
    // Claim interface and set alternate setting
    handle.claim_interface(endpoint.iface).map_err(QckError::UsbError)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting).map_err(QckError::UsbError)?;
    
    Ok(())
}

/// Sends an acknowledgment command to the device
fn send_ack<T: UsbContext>(handle: &mut DeviceHandle<T>) -> Result<usize> {
    // Create a zeroed buffer with the ACK command byte
    let mut ack = [0u8; 64];
    ack[0] = CMD_ACK;

    // Send the command and return the number of bytes written
    handle
        .write_control(REQUEST_TYPE, REQUEST, ACK_VALUE, INDEX, &ack, TIMEOUT)
        .map_err(QckError::UsbError)
}

/// Sets the light level on the device
fn set_light<T: UsbContext>(light: u8, handle: &mut DeviceHandle<T>) -> Result<()> {
    // Create a zeroed buffer with the light command bytes
    let mut command = [0u8; 64];
    command[0] = CMD_LIGHT;  // Command identifier
    command[2] = light;      // Light level

    // Send the command
    handle
        .write_control(REQUEST_TYPE, REQUEST, LIGHT_VALUE, INDEX, &command, TIMEOUT)
        .map(|_| ())
        .map_err(QckError::UsbError)
}

/// Sets the colors on the device
fn set_color<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    color1: Color,
    color2: Color,
) -> Result<()> {
    // Create a zeroed buffer for the color command
    let mut command = [0u8; 524];
    
    // Set header bytes
    command[0] = CMD_COLOR;       // Command identifier
    command[2] = CMD_COLOR_COUNT; // Number of colors
    
    // Set first color data
    command[4] = color1.r;
    command[5] = color1.g;
    command[6] = color1.b;
    command[7] = ALPHA_FULL;       // Alpha (full opacity)
    command[8] = EFFECT_SPEED;     // Effect speed
    command[9] = EFFECT_INTENSITY; // Effect intensity
    command[13] = EFFECT_FLAG;     // Effect flag
    
    // Set second color data
    command[16] = color2.r;
    command[17] = color2.g;
    command[18] = color2.b;
    command[19] = ALPHA_FULL;       // Alpha (full opacity)
    command[20] = EFFECT_SPEED;     // Effect speed
    command[21] = EFFECT_INTENSITY; // Effect intensity
    command[24] = EFFECT_FLAG;      // Effect flag 1
    command[25] = EFFECT_FLAG;      // Effect flag 2
    command[27] = EFFECT_FLAG;      // Effect flag 3

    // Send the command
    handle
        .write_control(REQUEST_TYPE, REQUEST, COLOR_VALUE, INDEX, &command, TIMEOUT)
        .map(|_| ())
        .map_err(QckError::UsbError)
}
