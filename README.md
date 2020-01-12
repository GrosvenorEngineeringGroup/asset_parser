# Asset Parser

A command-line tool to check the validity of asset/sensor JSON files. It will
write pretty-formatted JSON files in the working directory. If any errors were found, it will print
the error messages to the terminal.

## Installation
1. Install rustup, which will install cargo.
1. `cd path/to/asset_parser`
1. `cargo install --force --path .`
1. Run:
    1. `asset_parser 'path/to/assets.json' 'path/to/sensors.json'`

# Units.txt License
The file `units.txt` was copied from Project Haystack, and that file is licensed under
the Academic Free License v3.0. A copy of the license can be found in
the `lic` folder.
