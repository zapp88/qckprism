

# QcKPrism

QcKPrism is a small cross-platform CLI utility for controlling the RGB
lighting of a SteelSeries QcK Prism Cloth XL mousepad. SteelSeries does
not provide an official Linux tool, so this project offers basic control
without the need for SteelSeries Engine.

## Supported device

Currently the project targets a single device:

- Vendor ID: `0x1038`
- Product ID: `0x150d`

Other SteelSeries devices have not been tested.

## Building

This repository contains only Rust code, so building the project is as
simple as:

```bash
cargo build --release
```

## Usage

```
qckprism [OPTIONS] --color1 <HEX> --color2 <HEX>
```

### Options

- `-a, --color1 <HEX>` – Sets LED1 color in hex (e.g. `FF00FF`)
- `-b, --color2 <HEX>` – Sets LED2 color in hex (e.g. `FF00FF`)
- `-l, --light <LIGHT>` – Sets light level (0‑255, defaults to 255)
- `-h, --help` – Prints help information
- `-V, --version` – Prints version information

The color flags are required and currently only static lighting is
supported.

## Known issues

### Linux

- Running the utility may require `sudo`. To allow regular users to
  access the device, create a udev rule at
  `/etc/udev/rules.d/75-qck-prism.rules` with the following contents:

  ```
  SUBSYSTEM=="usb", ATTRS{idVendor}=="1038", ATTRS{idProduct}=="150d", TAG+="uaccess"
  ```

  Reload udev rules or replug the device after creating the file.

### macOS

- Although the utility compiles on macOS, it typically fails with
  `Error: Access` because the HID device is claimed by the kernel. A
  custom kernel extension would be required to claim the device before
  the system does. See [tessel/node-usb#30](https://github.com/tessel/node-usb/issues/30)
  for more information.
