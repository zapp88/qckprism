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

/// Represents a USB endpoint configuration
#[derive(Debug, Clone)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
}

/// Command to send to the QCK Prism XL device
pub struct Command {
    pub light_level: u8,
    pub first_color: Color,
    pub second_color: Color,
}

/// RGB color representation
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Sends a command to the QCK Prism XL device
pub fn send_to_device(command: Command) -> Result<()> {
    // Initialize USB context
    let mut context = Context::new().map_err(QckError::UsbError)?;
    
    // Open the device
    let (mut device, mut handle) = open_device(&mut context, VID, PID)?;

    // Find and configure endpoints
    let endpoints = find_readable_endpoints(&mut device)?;
    let endpoint = endpoints.get(0).ok_or(QckError::NoEndpointsFound)?;

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

    // Send commands to the device
    set_light(command.light_level, &mut handle)?;
    set_color(&mut handle, command.first_color, command.second_color)?;
    send_ack(&mut handle)?;

    // Cleanup: release interface and reattach kernel driver if needed
    handle.release_interface(endpoint.iface).map_err(QckError::UsbError)?;
    if has_kernel_driver {
        handle.attach_kernel_driver(endpoint.iface).map_err(QckError::UsbError)?;
    }
    
    Ok(())
}

/// Opens the USB device with the specified vendor and product IDs
fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Result<(Device<T>, DeviceHandle<T>)> {
    context.devices()
        .map_err(QckError::UsbError)?
        .iter()
        .filter_map(|device| {
            let device_desc = device.device_descriptor().ok()?;
            if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
                device.open().ok().map(|handle| (device, handle))
            } else {
                None
            }
        })
        .next()
        .ok_or(QckError::DeviceNotFound(vid, pid))
}

/// Finds all readable endpoints on the device
fn find_readable_endpoints<T: UsbContext>(device: &mut Device<T>) -> Result<Vec<Endpoint>> {
    let device_desc = device.device_descriptor().map_err(QckError::UsbError)?;
    let mut endpoints = vec![];

    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue, // Skip if config descriptor cannot be read
        };

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                for _endpoint_desc in interface_desc.endpoint_descriptors() {
                    endpoints.push(Endpoint {
                        config: config_desc.number(),
                        iface: interface_desc.interface_number(),
                        setting: interface_desc.setting_number(),
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
    if active_config != endpoint.config {
        handle.set_active_configuration(endpoint.config).map_err(QckError::UsbError)?;
    }
    
    handle.claim_interface(endpoint.iface).map_err(QckError::UsbError)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting).map_err(QckError::UsbError)?;
    
    Ok(())
}

/// Sends an acknowledgment command to the device
fn send_ack<T: UsbContext>(handle: &mut DeviceHandle<T>) -> Result<usize> {
    // Values are picked directly from the captured packet
    const ACK: [u8; 64] = [
        0x0d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];

    let bytes_written = handle
        .write_control(REQUEST_TYPE, REQUEST, ACK_VALUE, INDEX, &ACK, TIMEOUT)
        .map_err(QckError::UsbError)?;
        
    Ok(bytes_written)
}

/// Sets the light level on the device
fn set_light<T: UsbContext>(light: u8, handle: &mut DeviceHandle<T>) -> Result<()> {
    let mut command = [0u8; 64];
    command[0] = 0x0c;
    command[2] = light;

    handle
        .write_control(REQUEST_TYPE, REQUEST, LIGHT_VALUE, INDEX, &command, TIMEOUT)
        .map_err(QckError::UsbError)?;

    Ok(())
}

/// Sets the colors on the device
fn set_color<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    color1: Color,
    color2: Color,
) -> Result<()> {
    let mut command = [0u8; 524];
    
    // Header and first color
    command[0] = 0x0e;
    command[2] = 0x02;
    command[4] = color1.r;
    command[5] = color1.g;
    command[6] = color1.b;
    command[7] = 0xff;
    command[8] = 0x32;
    command[9] = 0xc8;
    command[13] = 0x01;
    
    // Second color
    command[16] = color2.r;
    command[17] = color2.g;
    command[18] = color2.b;
    command[19] = 0xff;
    command[20] = 0x32;
    command[21] = 0xc8;
    command[24] = 0x01;
    command[25] = 0x01;
    command[27] = 0x01;

    handle
        .write_control(REQUEST_TYPE, REQUEST, COLOR_VALUE, INDEX, &command, TIMEOUT)
        .map_err(QckError::UsbError)?;

    Ok(())
}
