## QcKPrism - A SteelSeries QcK Prism Cloth XL controll utility

Since SteelSeries does not provide ways to control their QcK Prism Cloth XL under linux (SteelSeries Engine Software is available only under Windows or MacOS).
I didn't want to stay stuck with default lighting, so i made this simple utility to allow basic RGB control. 
This cli-utility was made with linux in mind, but since it's 100% rust code - it works on windows as well. 

I don't have any way to test if it works on other VID/PID devices from steelseries. For now i only support only one device :

    > VendorID: 0x1038
    > ProductID : 0x150d

Known issues:
* Linux:
    * Running utility may require 'sudo'. If you don't want to run utility with sudo you can create an udev rule, creating file /etc/udev/rules.d/75-qck-prism.rules with content : SUBSYSTEM=="usb", ATTRS{idVendor}=="1038", ATTRS{idProduct}=="150d", TAG+="uaccess" - this will give acess to our device to normal processes.
    * You can do that running command: 
    > sudo echo 'SUBSYSTEM=="usb", ATTRS{idVendor}=="1038", ATTRS{idProduct}=="150d", TAG+="uaccess"' > /etc/udev/rules.d/75-qck-prism.rules

* MacOS:
    * While utility should work in theory on MacOS in most cases you will get Error:Access, this is a result of how Mac handles HID devices. Our app can't 'claim' 
        'device' since its claimed by kernel, on linux we can circumvent that detaching it from kernel but on MacOS there is no such option. Only way to make it work on mac would be to write our own
        .kext (kernel extension) that would clame device before the system. There are quite few ongoing issus regarding that matter : eg. https://github.com/tessel/node-usb/issues/30
    

USAGE:

     qckprism.exe [OPTIONS] --color1 <COLOR1> --color2 <COLOR2>

FLAGS:

     -h, --help       Prints help information
     -V, --version    Prints version information

OPTIONS:

     -a, --color1 <COLOR1>    Sets LED1 color in hex (eg. FF00FF)
     -b, --color2 <COLOR2>    Sets LED2 color in hex (eg. FF00FF)
     -l, --light <LIGHT>      Sets light level (0-255)
  
 LED flags are required. For now only static light is supported.
