extern crate clap;
extern crate hex;
use std::time::Duration;

use clap::{App, Arg};
use rusb::{Context, Device, DeviceHandle, Result, UsbContext};

const VID: u16 = 0x1038;
const PID: u16 = 0x150d;

#[derive(Debug)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}

fn main() -> Result<()> {
    let matches = App::new("SteelSeries QCK Prism XL RGB driver")
        .version("0.1")
        .author("Jakub Maciej <zapp88@gmail.com>")
        .about("This utility allows you to control RGB lighting on youre QCK Prism XL")
        .arg(
            Arg::with_name("light")
                .short("l")
                .long("light")
                .value_name("LIGHT")
                .help("Sets light level (0-255)")
                .takes_value(true),
                
        ).arg(
            Arg::with_name("color1")
                .short("a")
                .long("color1")
                .value_name("COLOR1")
                .help("Sets LED1 color in hex (eg. FF00FF)")
                .required(true)
                .takes_value(true),
                
        ).arg(
            Arg::with_name("color2")
                .short("b")
                .long("color2")
                .value_name("COLOR2")
                .help("Sets LED2 color in hex (eg. FF00FF)")
                .required(true)
                .takes_value(true),
                
        )
        .get_matches();

    
    let light_value = matches.value_of("light").unwrap_or("255");
    let light_num= light_value.parse::<i32>().expect("Light must be value between 0 and 255");
    if light_num > 255 || light_num < 0 {
        panic!("Light must be value between 0 and 255");
    }   


    let color1_value = matches.value_of("color1").unwrap_or("FFFFFF");
    let color1_num = hex::decode(color1_value).expect("Color1 must be hex value between 0000000 and FFFFFF");
    if color1_num.len() != 3{
        panic!("Color1 must be hex value between 0000000 and FFFFFF");
    }

    let color2_value = matches.value_of("color2").unwrap_or("FFFFFF");
    let color2_num = hex::decode(color2_value).expect("Color2 must be hex value between 0000000 and FFFFFF");
    if color2_num.len() != 3{
        panic!("Color2 must be hex value between 0000000 and FFFFFF");
    }


    let mut context = Context::new()?;
    let (mut device, mut handle) =
        open_device(&mut context, VID, PID).expect("Failed to open USB device");

    let endpoints = find_readable_endpoints(&mut device)?;
    let endpoint = endpoints
        .get(0)
        .expect("No Configurable endpoint found on device");

    let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
        Ok(true) => {
            handle.detach_kernel_driver(endpoint.iface)?;
            true
        }
        _ => false,
    };

    configure_endpoint(&mut handle, &endpoint)?;

    set_light(light_num as u8,&mut handle)?;

    let r1=*(color1_num.get(0).unwrap());
    let g1=*(color1_num.get(1).unwrap()); 
    let b1=*(color1_num.get(2).unwrap()); 

    let r2=*(color2_num.get(0).unwrap());
    let g2=*(color2_num.get(1).unwrap()); 
    let b2=*(color2_num.get(2).unwrap()); 


    set_color(&mut handle, [r1, g1, b1], [r2, g2, b2])?;

    handle.release_interface(endpoint.iface)?;
    if has_kernel_driver {
        handle.attach_kernel_driver(endpoint.iface)?;
    }
    Ok(())
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, handle)),
                Err(_) => continue,
            }
        }
    }

    None
}

fn find_readable_endpoints<T: UsbContext>(device: &mut Device<T>) -> Result<Vec<Endpoint>> {
    let device_desc = device.device_descriptor()?;
    let mut endpoints = vec![];
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    endpoints.push(Endpoint {
                        config: config_desc.number(),
                        iface: interface_desc.interface_number(),
                        setting: interface_desc.setting_number(),
                        address: endpoint_desc.address(),
                    });
                }
            }
        }
    }

    Ok(endpoints)
}

fn configure_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) -> Result<()> {
    handle.set_active_configuration(endpoint.config)?;
    handle.claim_interface(endpoint.iface)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting)
}

fn send_ack<T: UsbContext>(handle: &mut DeviceHandle<T>) -> Result<usize> {
    let timeout = Duration::from_secs(1);

    // values are picked directly from the captured packet
    const REQUEST_TYPE: u8 = 0x21;
    const REQUEST: u8 = 0x09;
    const VALUE: u16 = 0x0200;
    const INDEX: u16 = 0x0000;

    const ACK: [u8; 64] = [
        0x0d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];

    handle.write_control(REQUEST_TYPE, REQUEST, VALUE, INDEX, &ACK, timeout)
}

fn set_light<T: UsbContext>(light: u8, handle: &mut DeviceHandle<T>) -> Result<()> {
    let command: [u8; 64] = [
        0x0c, 0x00, light, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00,
    ];

    let timeout = Duration::from_secs(1);

    // values are picked directly from the captured packet
    const REQUEST_TYPE: u8 = 0x21;
    const REQUEST: u8 = 0x09;
    const VALUE: u16 = 0x0200;
    const INDEX: u16 = 0x0000;

    handle.write_control(REQUEST_TYPE, REQUEST, VALUE, INDEX, &command, timeout)?;

    send_ack(handle)?;

    Ok(())
}

fn set_color<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    color1: [u8; 3],
    color2: [u8; 3],
) -> Result<()> {
    let command: [u8; 524] = [
        0x0e, 0x00, 0x02, 0x00, color1[0], color1[1], color1[2], 0xff, 0x32, 0xc8, 0x00, 0x00,
        0x00, 0x01, 0x00, 0x00, color2[0], color2[1], color2[2], 0xff, 0x32, 0xc8, 0x00, 0x00,
        0x01, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    let timeout = Duration::from_secs(1);

    // values are picked directly from the captured packet
    const REQUEST_TYPE: u8 = 0x21;
    const REQUEST: u8 = 0x09;
    const VALUE: u16 = 0x0300;
    const INDEX: u16 = 0x0000;

    handle.write_control(REQUEST_TYPE, REQUEST, VALUE, INDEX, &command, timeout)?;

    send_ack(handle)?;

    Ok(())
}
