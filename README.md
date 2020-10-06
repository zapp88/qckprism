## QcKPrism - A SteelSeries QcK Prism Cloth XL controll utility

Since SteelSeries does not provide ways to control their QcK Prism Cloth XL under linux (SteelSeries Engine Software is available only under Windows or MacOS).
I didn't want to stay stuck with default lighting, so i made this simple utility to allow basic RGB control. 
This cli-utility was made with linux in mind, but since it's 100% rust code - it works on windows as well. 

I don't have any way to test if it works on other VID/PID devices from steelseries. For now i only support only one device :

    > VendorID: 0x1038
    > ProductID : 0x150d

USAGE:

    > qckprism.exe [OPTIONS] --color1 <COLOR1> --color2 <COLOR2>

FLAGS:

    > -h, --help       Prints help information
    > -V, --version    Prints version information

OPTIONS:

    > -a, --color1 <COLOR1>    Sets LED1 color in hex (eg. FF00FF)
    > -b, --color2 <COLOR2>    Sets LED2 color in hex (eg. FF00FF)
    > -l, --light <LIGHT>      Sets light level (0-255)
  
 LED flags are required. For now only static light is supported.
