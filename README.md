# Asset Parser

A command-line tool to check the validity of asset/sensor JSON files. It will
write pretty-formatted JSON files in the working directory. If any errors were found, it will print
the error messages to the terminal.

## Installation
1. Install rustup, which will install cargo.
1. `cd path/to/asset_parser`
1. `cargo install --force --path .`
1. Run `asset_parser 'path/to/assets.json' 'path/to/sensors.json'`

## Cross-compilation
To compile this project for a Linux machine (e.g. a machine which runs GitHub Actions CI):
1. Install rustup
1. Install cross (https://github.com/rust-embedded/cross)
1. To avoid a current bug in rustup/cross:
    1. Run `rustup set profile minimal`
        * Currently, the default profile causes an error when running the `cross <args>` command below.
    1. Run `rustup target install x86_64-unknown-linux-gnu`
        * This will install the minimal target we want to use in the subsequent `cross <args>` command.
1. Run `cross build --release --target x86_64-unknown-linux-gnu`
    * This will build a binary located at `./target/x86_64-unknown-linux-gnu/release/asset_parser`.
1. Copy that binary file, and rename it to `asset_parser_x86_64-unknown-linux-gnu`.
1. Create a new release for this repository on GitHub's website.
1. Upload that binary file to the GitHub release. The gegCoreLibExt repository uses this binary in its CI scripts.
1. Change the rustup profile back to default with `rustup set profile default`

## src/arms-asset-type-ids.txt
This file enumerates the Asset Type IDs in ARMS. This file should be updated manually whenever new asset types are added to ARMS.

## src/units.txt
This file contains all the valid units used by Project Haystack. This file should be update manually whenever new units are added to Project Haystack.

# Units.txt License
The file `units.txt` was copied from Project Haystack, and that file is licensed under
the Academic Free License v3.0. A copy of the license can be found in
the `lic` folder.
