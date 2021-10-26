# Asset Parser

A command-line tool to check the validity of asset/sensor JSON files. It will
write two formatted JSON files in the current working directory,
new_assets.json and new_sensors.json. If any errors were found, it will print
the error messages to the terminal.

This tool is used in a CI/CD script in the gegCoreLibExt repository to
validate the contents of the assets library, as well as maintain
a consistent format for the assets library.

## Installation
1. Install [rustup](https://rustup.rs), which will install the Rust programming language and the cargo command line tool.
1. Navigate to the asset_parser directory: `cd path/to/asset_parser`
1. Install asset_parser: `cargo install --force --path .`
1. Run the asset_parser command line tool with: `asset_parser 'path/to/assets.json' 'path/to/sensors.json'`
    * These two files can be found at https://github.com/GrosvenorEngineeringGroup/gegCoreLibExt/tree/master/asset-library

## Cross-compilation for GitHub Actions
To compile this project for a Linux machine (for example, a GitHub Actions CI/CD server):
1. Install rustup (see above).
1. Install cross (https://github.com/rust-embedded/cross).
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
1. Update the GitHub Actions CI/CD [script in the gegCoreLibExt repository](https://github.com/GrosvenorEngineeringGroup/gegCoreLibExt/tree/master/.github/workflows) by updating the version number of asset_parser to match this repository.
1. Change the rustup profile back to default with `rustup set profile default`

## src/arms-asset-type-ids.txt
This file enumerates the Asset Type IDs in ARMS. This file should be updated manually whenever new asset types are added to ARMS.

## src/units.txt
This file contains all the valid units used by Project Haystack. This file should be update manually whenever new units are added to Project Haystack.

# Units.txt License
The file `units.txt` is from [Project Haystack](https://project-haystack.org/download/units.txt),
and that file is licensed under
the Academic Free License v3.0. A copy of the license can be found in
the `lic` folder.
