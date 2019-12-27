use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;

const UNITS_TXT: &str = include_str!("units.txt");

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Sensor {
    id: String,
    display_name: String,
    skyspark_marker_tags: Vec<String>,
    #[serde(rename = "type")]
    sensor_type: SensorType,
    unit: Option<String>,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
enum SensorType {
    Bool,
    Numeric,
    String,
}

fn main() {
    let mode = parse_args();
    match mode {
        Mode::ParseSensors { filepath } => parse_sensors(&filepath),
        Mode::ParseAssets {
            filepath: _filepath,
        } => println!("TODO"),
    }
    ()
}

fn parse_sensors(filepath: &str) {
    let units = units();
    let file_contents = fs::read_to_string(filepath).unwrap();
    let raw_sensors: Vec<Sensor> =
        serde_json::from_str(&file_contents).unwrap();
    let sensors = clean_raw_sensors(raw_sensors);

    println!("{}", serde_json::to_string_pretty(&sensors).unwrap());

    for sensor in &sensors {
        let id = &sensor.id;
        let tags = &sensor.skyspark_marker_tags;
        let unit = &sensor.unit;

        if id.is_empty() {
            println!("A sensor has an empty id");
        }
        if sensor.display_name.is_empty() {
            println!("Sensor id={} has an empty display name", id);
        }
        if tags.is_empty() {
            println!("Sensor id={} has no SkySpark marker tags", id);
        }
        for tag in tags {
            if !is_tag_name(tag) {
                println!(
                    "Sensor id={} has an invalid SkySpark marker tag '{}'",
                    id, tag
                );
            }
        }
        if sensor.sensor_type == SensorType::Numeric {
            match unit {
                Some(unit) => {
                    if !units.contains(unit) {
                        println!("Sensor id={} has an invalid unit", id);
                    }
                }
                None => (), // It's ok to have no unit, just highly uncommon.
            }
        } else {
            if unit.is_some() {
                println!("Sensor id={} has a unit but is not numeric", id);
            }
        }
    }

    if sensors.len() != unique_ids_count(&sensors) {
        println!("Some sensor ids are not unique")
    }
}

fn unique_ids_count(sensors: &[Sensor]) -> usize {
    let mut ids: Vec<String> =
        sensors.iter().map(|sensor| sensor.id.to_owned()).collect();
    ids.sort();
    ids.dedup();
    ids.len()
}

fn clean_raw_sensors(raw_sensors: Vec<Sensor>) -> Vec<Sensor> {
    let mut sensors: Vec<Sensor> = raw_sensors
        .into_iter()
        .map(|raw_sensor| {
            let mut cleaned_tags: Vec<String> = raw_sensor
                .skyspark_marker_tags
                .into_iter()
                .map(|tag| tag.trim().to_owned())
                .collect();
            cleaned_tags.sort();

            Sensor {
                id: raw_sensor.id.trim().to_owned(),
                display_name: raw_sensor.display_name.trim().to_owned(),
                skyspark_marker_tags: cleaned_tags,
                sensor_type: raw_sensor.sensor_type,
                unit: raw_sensor
                    .unit
                    .map(|unit_str| unit_str.trim().to_owned()),
            }
        })
        .collect();
    sensors.sort_by(|a, b| a.id.cmp(&b.id));
    sensors
}

fn parse_args() -> Mode {
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        print_help();
    } else {
        let command = args.remove(1);
        let filepath = args.remove(1);
        match command.as_ref() {
            "sensors" => Mode::ParseSensors { filepath },
            "assets" => Mode::ParseAssets { filepath },
            _ => print_help(),
        }
    }
}

fn print_help() -> ! {
    println!("Could not parse arguments.");
    println!("Example usages:");
    println!("    asset_parser sensors '/path/to/sensors.json'");
    println!("    asset_parser assets '/path/to/assets.json'");
    std::process::exit(0);
}

enum Mode {
    ParseSensors { filepath: String },
    ParseAssets { filepath: String },
}

/// Return true if the string is a valid SkySpark tag name.
pub fn is_tag_name<T: AsRef<str>>(s: T) -> bool {
    let s = s.as_ref();
    if s.is_empty() {
        false
    } else {
        let chars = s.chars().enumerate();
        let mut is_tag_name = true;
        for (index, c) in chars {
            if index == 0 {
                if !c.is_ascii_lowercase() {
                    is_tag_name = false;
                    break;
                }
            } else if !(c.is_ascii_alphanumeric() || c == '_') {
                is_tag_name = false;
                break;
            };
        }
        is_tag_name
    }
}

fn units() -> HashSet<String> {
    let mut units = HashSet::new();
    for line in UNITS_TXT.lines() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with("--") {
            units.extend(line.split(",").map(|s| s.to_owned()));
        }
    }
    units
}
