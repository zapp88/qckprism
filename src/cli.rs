extern crate clap;
extern crate hex;

use crate::qck;
use clap::{App, Arg};

pub struct Args {
    pub first_color: qck::Color,
    pub second_color: qck::Color,
    pub light_level: u8,
}

pub fn fetch_cli_args() -> Args {
    let matches = App::new("SteelSeries QCK Prism XL RGB driver")
        .version("0.1")
        .author("Jakub Maciej <zapp88@gmail.com>")
        .about("This utility allows you to control RGB lighting on youre QCK Prism XL")
        .arg(
            Arg::with_name("light")
                .short('l')
                .long("light")
                .value_name("LIGHT")
                .help("Sets light level (0-255)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("color1")
                .short('a')
                .long("color1")
                .value_name("COLOR1")
                .help("Sets LED1 color in hex (eg. FF00FF)")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("color2")
                .short('b')
                .long("color2")
                .value_name("COLOR2")
                .help("Sets LED2 color in hex (eg. FF00FF)")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let light_value = matches.value_of("light").unwrap_or("255");
    let light_num = light_value
        .parse::<i32>()
        .expect("Light must be value between 0 and 255");
    if light_num > 255 || light_num < 0 {
        panic!("Light must be value between 0 and 255");
    }

    let color1_value = matches.value_of("color1").unwrap_or("FFFFFF");
    let color1_num =
        hex::decode(color1_value).expect("Color1 must be hex value between 0000000 and FFFFFF");
    if color1_num.len() != 3 {
        panic!("Color1 must be hex value between 0000000 and FFFFFF");
    }

    let color2_value = matches.value_of("color2").unwrap_or("FFFFFF");
    let color2_num =
        hex::decode(color2_value).expect("Color2 must be hex value between 0000000 and FFFFFF");
    if color2_num.len() != 3 {
        panic!("Color2 must be hex value between 0000000 and FFFFFF");
    }

    let r1 = *(color1_num.get(0).unwrap());
    let g1 = *(color1_num.get(1).unwrap());
    let b1 = *(color1_num.get(2).unwrap());

    let r2 = *(color2_num.get(0).unwrap());
    let g2 = *(color2_num.get(1).unwrap());
    let b2 = *(color2_num.get(2).unwrap());

    return Args {
        first_color: qck::Color {
            r: r1,
            g: g1,
            b: b1,
        },
        second_color: qck::Color {
            r: r2,
            g: g2,
            b: b2,
        },
        light_level: light_num as u8,
    };
}
