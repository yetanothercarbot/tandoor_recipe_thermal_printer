# Receipt Recipe Printer for Tandoor

This is a basic CLI app that pulls recipes from an instance of [Tandoor Recipes](https://docs.tandoor.dev/) and prints it on an ESC/POS compatible printer.

![Example receipt printed](sample.png)


## Why?
Mainly for fun :)

It does have a few benefits over Tandoor's native printing output:
- Space (the image tends to be very large)
- Entertainment value
- Single piece of paper - no more shuffling papers or flipping over to follow instructions.

## Usage
Binaries for Linux can be downloaded from the releases tab. There are three required parameters:

1. **Address of Tandoor Instance with protocol**: e.g. https://recipes.example.com
2. **Authentication token** with read permission. This can be created in Settings > API > New.
3. **Recipe ID(s)**: One or more IDs of the recipe to print

Additional flags are also available:

| Flag                         | Valid Options                     | Default        | Description                                                                                                                                                                                                                                                 |
|------------------------------|-----------------------------------|----------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `--printer-path`, `-p`       | Path to printer control file      | `/dev/usb/lp0` | The location on which the printer is made available by the kernel.                                                                                                                                                                                          |
| `--ingredient-display`, `-i` | `both`, `summary`, `step`, `none` | `step`         | Choose how to display ingredients: As a summary, similar to a traditional recipe (`summary`), at each step (`step`), both or none.                                                                                                                          |
| ``--qr``                     |                                   |                | Add a QR code with the link to the recipe on Tandoor                                                                                                                                                                                                        |
| `--cut-mode`                 | none, pause, partial, full        | `full`         | When batch printing, select the cut at the end of the recipe. `none` will simply print all recipes without a cut, `pause` will allow the user to tear the recipe off for printers without a cutter, `partial` and `full` use the printer's built-in cutter. |
| `--columns`                  |                                   | 42             | Enter the width of your printer in characters, or 0 to disable word wrapping.                                                                                                                                                                               |


## Compatibility

### Hardware
The application has been tested against a Partner RP-320 printer. It is expected that the application works on other USB-connected ESC/POS-compatible printers. Serial/parallel printers may work, given the correct path (using the `--printer-path` flag). Networked printers are not supported - I do not have one to test. 

### Software
The pre-built binaries are for Linux on x86-64 and aarch64. They are compiled statically so should work across all distributions. Other OSes have not been tested at this time.  

## Building
This application is built with cargo. Simply run `cargo build` to receive a binary.

## License
Affero GPL 3.0
