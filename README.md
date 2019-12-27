# Asset Parser

A command-line tool to check the validity of asset/sensor JSON files. It will
pretty-print JSON to the terminal. If any errors were found, it will print
the error messages to the terminal.

## Installation
1. Install rustup, which will install cargo.
1. `cd path/to/asset_parser`
1. `cargo install --force --path .`
1. Check either assets or sensors:
    1. `asset_parser sensors 'path/to/sensors.json'`
    1. `asset_parser assets 'path/to/assets.json'`

# License
The file `units.txt` was copied from Project Haystack, and is licensed under
the Academic Free License v3.0. A copy of the license can be found in
the `lic` folder.