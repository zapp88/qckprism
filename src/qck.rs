use crate::error::{QckError, Result};
use rusb::{Context, Device, DeviceHandle, UsbContext};
use std::time::Duration;

/// SteelSeries QCK Prism XL device identifiers
const VID: u16 = 0x1038; // Vendor ID
const PID: u16 = 0x150d; // Product ID

/// USB control transfer constants
const REQUEST_TYPE: u8 = 0x21;
const REQUEST: u8 = 0x09;
const INDEX: u16 = 0x0000;
const TIMEOUT: Duration = Duration::from_secs(1);

/// USB command values
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
enum CommandValue {
    Light = 0x0200,
    Color = 0x0300,
}

/// Represents a USB endpoint configuration
#[derive(Debug, Clone, Copy)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
}

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Create a new RGB color
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    
    /// Create a color from RGB bytes
    pub fn from_rgb_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 3 {
            return Err(QckError::InvalidColorFormat("Invalid RGB byte length".to_string()));
        }
        
        Ok(Self {
            r: bytes[0],
            g: bytes[1],
            b: bytes[2],
        })
    }
}

/// Command to send to the QCK Prism XL device
#[derive(Debug, Clone)]
pub struct Command {
    pub light_level: u8,
    pub first_color: Color,
    pub second_color: Color,
}

impl Default for Command {
    fn default() -> Self {
        Self {
            light_level: 255,
            first_color: Color::new(255, 0, 0),  // Default red
            second_color: Color::new(0, 0, 255), // Default blue
        }
    }
}

/// QCK Prism XL USB device wrapper
pub struct QckDevice<T: UsbContext> {
    #[allow(dead_code)]
    device: Device<T>,  // Keeping the device reference alive while the handle is in use
    handle: DeviceHandle<T>,
    endpoint: Endpoint,
    has_kernel_driver: bool,
}

impl<T: UsbContext> QckDevice<T> {
    /// Create a new QCK device instance
    pub fn new(context: &mut T) -> Result<Self> {
        // Open the device
        let (mut device, mut handle) = Self::open_device(context, VID, PID)?;

        // Find endpoints
        let endpoints = Self::find_readable_endpoints(&mut device)?;
        let endpoint = *endpoints.get(0).ok_or(QckError::NoEndpointsFound)?;

        // Check kernel driver
        let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
            Ok(true) => {
                handle.detach_kernel_driver(endpoint.iface).map_err(QckError::UsbError)?;
                true
            }
            _ => false,
        };

        // Configure endpoint
        Self::configure_endpoint(&mut handle, &endpoint)?;
        
        Ok(Self {
            device,
            handle,
            endpoint,
            has_kernel_driver,
        })
    }

    /// Send a command to the device
    pub fn send_command(&mut self, command: &Command) -> Result<()> {
        self.set_light(command.light_level)?;
        self.set_color(command.first_color, command.second_color)?;
        self.send_ack()?;
        Ok(())
    }
    
    /// Close the device properly
    pub fn close(self) -> Result<()> {
        // Release interface
        self.handle.release_interface(self.endpoint.iface).map_err(QckError::UsbError)?;
        
        // Reattach kernel driver if needed
        if self.has_kernel_driver {
            self.handle.attach_kernel_driver(self.endpoint.iface).map_err(QckError::UsbError)?;
        }
        
        Ok(())
    }

    /// Opens the USB device with the specified vendor and product IDs
    fn open_device(
        context: &mut T,
        vid: u16,
        pid: u16,
    ) -> Result<(Device<T>, DeviceHandle<T>)> {
        let devices = context.devices().map_err(QckError::UsbError)?;

        // Find our device by VID/PID
        for device in devices.iter() {
            if let Ok(desc) = device.device_descriptor() {
                if desc.vendor_id() == vid && desc.product_id() == pid {
                    if let Ok(handle) = device.open() {
                        return Ok((device, handle));
                    }
                }
            }
        }

        Err(QckError::DeviceNotFound(vid, pid))
    }

    /// Finds all readable endpoints on the device
    fn find_readable_endpoints(device: &mut Device<T>) -> Result<Vec<Endpoint>> {
        let device_desc = device.device_descriptor().map_err(QckError::UsbError)?;
        let mut endpoints = Vec::new();
        
        // Iterate through all configurations, interfaces, and endpoints
        for n in 0..device_desc.num_configurations() {
            if let Ok(config_desc) = device.config_descriptor(n) {
                for interface in config_desc.interfaces() {
                    for interface_desc in interface.descriptors() {
                        // If the interface has any endpoints, it's usable for us
                        let has_endpoints = interface_desc.endpoint_descriptors().next().is_some();
                        if has_endpoints {
                            endpoints.push(Endpoint {
                                config: config_desc.number(),
                                iface: interface_desc.interface_number(),
                                setting: interface_desc.setting_number(),
                            });
                        }
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
    fn configure_endpoint(
        handle: &mut DeviceHandle<T>,
        endpoint: &Endpoint,
    ) -> Result<()> {
        // Avoid "device busy" error by checking current config
        let active_config = handle.active_configuration().map_err(QckError::UsbError)?;
        if active_config != endpoint.config {
            handle.set_active_configuration(endpoint.config).map_err(QckError::UsbError)?;
        }
        
        handle.claim_interface(endpoint.iface).map_err(QckError::UsbError)?;
        handle.set_alternate_setting(endpoint.iface, endpoint.setting).map_err(QckError::UsbError)?;
        
        Ok(())
    }

    /// Sends an acknowledgment command to the device
    fn send_ack(&mut self) -> Result<usize> {
        // Values from captured packet - just the command byte (0x0d) and zeros
        let mut ack = [0u8; 64];
        ack[0] = 0x0d;

        let bytes_written = self.handle
            .write_control(
                REQUEST_TYPE, 
                REQUEST, 
                CommandValue::Light as u16, 
                INDEX, 
                &ack, 
                TIMEOUT
            )
            .map_err(QckError::UsbError)?;
            
        Ok(bytes_written)
    }

    /// Sets the light level on the device
    fn set_light(&mut self, light_level: u8) -> Result<()> {
        let mut command = [0u8; 64];
        command[0] = 0x0c;  // Command identifier
        command[2] = light_level;

        self.handle
            .write_control(
                REQUEST_TYPE, 
                REQUEST, 
                CommandValue::Light as u16, 
                INDEX, 
                &command, 
                TIMEOUT
            )
            .map_err(QckError::UsbError)?;

        Ok(())
    }

    /// Sets the colors on the device
    fn set_color(&mut self, color1: Color, color2: Color) -> Result<()> {
        let mut command = [0u8; 524];
        
        // Header and first color
        command[0] = 0x0e;  // Command identifier
        command[2] = 0x02;  // Number of colors
        
        // First color configuration
        command[4] = color1.r;
        command[5] = color1.g;
        command[6] = color1.b;
        command[7] = 0xff;  // Alpha (always 255)
        command[8] = 0x32;  // Speed parameter 1
        command[9] = 0xc8;  // Speed parameter 2
        command[13] = 0x01; // Effect type
        
        // Second color configuration
        command[16] = color2.r;
        command[17] = color2.g;
        command[18] = color2.b;
        command[19] = 0xff; // Alpha (always 255)
        command[20] = 0x32; // Speed parameter 1
        command[21] = 0xc8; // Speed parameter 2
        command[24] = 0x01; // Effect type
        command[25] = 0x01; // Additional parameter
        command[27] = 0x01; // Additional parameter

        self.handle
            .write_control(
                REQUEST_TYPE, 
                REQUEST, 
                CommandValue::Color as u16, 
                INDEX, 
                &command, 
                TIMEOUT
            )
            .map_err(QckError::UsbError)?;

        Ok(())
    }
}

/// Sends a command to the QCK Prism XL device
pub fn send_to_device(command: Command) -> Result<()> {
    // Initialize USB context
    let mut context = Context::new().map_err(QckError::UsbError)?;
    
    // Open and configure the device
    let mut device = QckDevice::new(&mut context)?;
    
    // Send the command
    device.send_command(&command)?;
    
    // Close the device properly
    device.close()?;
    
    Ok(())
}
