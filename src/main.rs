use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;

const UNITS_TXT: &str = include_str!("units.txt");

trait HasId {
    fn id(&self) -> String;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Sensor {
    id: String,
    display_name: String,
    skyspark_marker_tags: Vec<String>,
    #[serde(rename = "type")]
    sensor_type: SensorType,
    unit: Option<String>,
}

impl HasId for Sensor {
    fn id(&self) -> String {
        self.id.to_owned()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum SensorType {
    Bool,
    Numeric,
    String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct Asset {
    id: String,
    display_name: String,
    skyspark_marker_tags: Vec<String>,
    mandatory_sensors: Vec<SensorInfo>,
    optional_sensors: Vec<SensorInfo>,
    arms_asset_type_ids: Vec<u32>,
}

impl HasId for Asset {
    fn id(&self) -> String {
        self.id.to_owned()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct SensorInfo {
    sensor_id: String,
    extra_skyspark_marker_tags: Vec<String>,
}

fn main() {
    let args = parse_args();
    let assets_file_contents =
        fs::read_to_string(args.assets_filepath).expect("Could not read assets file to string");
    let sensors_file_contents =
        fs::read_to_string(args.sensors_filepath).expect("Could not read sensors file to string");
    let assets = parse_assets(&assets_file_contents);
    let sensors = parse_sensors(&sensors_file_contents);

    let sensor_errs = get_sensor_errors(&sensors);
    if !sensor_errs.is_empty() {
        for err in sensor_errs {
            println!("Sensor {}: {}", err.sensor_id, err.msg);
        }
        std::process::exit(1);
    }

    let asset_errs = get_asset_errors(&assets, &sensors_to_sensor_map(sensors));
    if !asset_errs.is_empty() {
        for err in asset_errs {
            println!("Asset {}: {}", err.asset_id, err.msg);
        }
        std::process::exit(1);
    }

    write_files(assets, sensors);
}

fn write_files(assets: Vec<Asset>, sensors: Vec<Sensor>) {
    let mut assets_output = File::create("new_assets.json").expect("Could not create new assets file");
    let assets = serde_json::to_value(assets.clone()).unwrap();
    write!(assets_output, "{}", to_pretty_string(&assets)).expect("Could not write to assets file");

    let mut sensors_output = File::create("new_sensors.json").expect("Could not create new sensors file");
    let sensors = serde_json::to_value(sensors.clone()).unwrap();
    write!(assets_output, "{}", to_pretty_string(&sensors)).expect("Could not write to sensors file");
}

fn all_ids_unique<T: HasId>(items: &[T]) -> bool {
    let mut unique_ids = HashSet::new();
    unique_ids.extend(items.iter().map(|item| item.id()));
    unique_ids.len() == items.len()
}

fn parse_assets(file_contents: &str) -> Vec<Asset> {
    let raw_assets: Vec<Asset> = serde_json::from_str(&file_contents).unwrap();
    clean_raw_assets(raw_assets)
}

struct AssetError {
    asset_id: String,
    msg: String,
}

impl AssetError {
    fn new<S, T>(asset_id: S, msg: T) -> Self
    where
        S: AsRef<str>,
        T: AsRef<str>,
    {
        AssetError {
            asset_id: asset_id.as_ref().to_string(),
            msg: msg.as_ref().to_string(),
        }
    }
}

fn get_asset_errors(
    assets: &Vec<Asset>,
    sensors: &HashMap<String, Sensor>,
) -> Vec<AssetError> {
    let mut errs = Vec::new();

    for asset in assets.iter() {
        let id = &asset.id;
        let asset_tags = &asset.skyspark_marker_tags;
        let mandatory_sensors = &asset.mandatory_sensors;
        let optional_sensors = &asset.optional_sensors;

        if id.is_empty() {
            errs.push(AssetError::new("?", "An asset has an empty id"));
        }
        if asset_tags.is_empty() {
            errs.push(AssetError::new(id, "No SkySpark marker tags"));
        }
        for asset_tag in asset_tags {
            if !is_tag_name(asset_tag) {
                errs.push(AssetError::new(
                    id,
                    format!("Invalid SkySpark marker tag '{}'", asset_tag),
                ));
            }
        }
        if asset.display_name.is_empty() {
            errs.push(AssetError::new(id, "Empty display name"));
        }
        if mandatory_sensors.is_empty() {
            errs.push(AssetError::new(id, "No mandatory sensors"));
        }
        errs.extend(check_asset_sensors(mandatory_sensors, sensors, id));
        errs.extend(check_asset_sensors(optional_sensors, sensors, id));
        if has_duplicate_sensor_ids(mandatory_sensors, optional_sensors) {
            errs.push(AssetError::new(id, "Duplicate sensor ids"));
        }
    }

    if !all_ids_unique(assets) {
        errs.push(AssetError::new("-", "Some assets have duplicate ids"));
    }

    errs
}

fn check_asset_sensors(
    sensor_infos: &Vec<SensorInfo>,
    sensors: &HashMap<String, Sensor>,
    asset_id: &str,
) -> Vec<AssetError> {
    let mut errs = Vec::new();

    for sensor_infos in sensor_infos {
        let sensor_id = &sensor_infos.sensor_id;
        let extra_sensor_tags = &sensor_infos.extra_skyspark_marker_tags;
        if sensor_id.is_empty() {
            errs.push(AssetError::new(asset_id, "Sensor has an empty id"));
        }
        match sensors.get(sensor_id) {
            None => errs.push(AssetError::new(
                asset_id,
                format!("No matching sensor with id '{}'", sensor_id),
            )),
            Some(sensor) => {
                let sensor_tags = &sensor.skyspark_marker_tags;
                let total_tags_count =
                    sensor_tags.len() + extra_sensor_tags.len();
                let mut unique_tags = HashSet::new();
                for tag in sensor_tags {
                    unique_tags.insert(tag.clone());
                }
                for tag in extra_sensor_tags {
                    unique_tags.insert(tag.clone());
                }
                if unique_tags.len() != total_tags_count {
                    errs.push(AssetError::new(
                        asset_id,
                        format!("Sensor '{}' has duplicate tags", sensor_id),
                    ));
                }

                for unique_tag in &unique_tags {
                    if !is_tag_name(unique_tag) {
                        errs.push(AssetError::new(
                            asset_id,
                            format!("Invalid tag {}", unique_tag),
                        ));
                    }
                }
            }
        }
    }

    errs
}

fn has_duplicate_sensor_ids(
    mandatory_sensors: &Vec<SensorInfo>,
    optional_sensors: &Vec<SensorInfo>,
) -> bool {
    let mut unique_sensor_ids = HashSet::new();
    unique_sensor_ids.extend(
        mandatory_sensors
            .iter()
            .map(|sensor| sensor.sensor_id.clone()),
    );
    unique_sensor_ids.extend(
        optional_sensors
            .iter()
            .map(|sensor| sensor.sensor_id.clone()),
    );
    unique_sensor_ids.len()
        != (mandatory_sensors.len() + optional_sensors.len())
}

fn sensors_to_sensor_map(sensors: Vec<Sensor>) -> HashMap<String, Sensor> {
    let mut map = HashMap::new();
    for sensor in sensors {
        map.insert(sensor.id.clone(), sensor);
    }
    map
}

fn parse_sensors(file_contents: &str) -> Vec<Sensor> {
    let raw_sensors: Vec<Sensor> =
        serde_json::from_str(&file_contents).unwrap();
    clean_raw_sensors(raw_sensors)
}

struct SensorError {
    sensor_id: String,
    msg: String,
}

impl SensorError {
    fn new<S, T>(sensor_id: S, msg: T) -> Self
    where
        S: AsRef<str>,
        T: AsRef<str>,
    {
        SensorError {
            sensor_id: sensor_id.as_ref().to_string(),
            msg: msg.as_ref().to_string(),
        }
    }
}

fn get_sensor_errors(sensors: &[Sensor]) -> Vec<SensorError> {
    let units = units();
    let mut errs = Vec::new();

    for sensor in sensors.iter() {
        let id = &sensor.id;
        let tags = &sensor.skyspark_marker_tags;
        let unit = &sensor.unit;

        if id.is_empty() {
            errs.push(SensorError::new("?", "A sensor has an empty id"));
        }
        if sensor.display_name.is_empty() {
            errs.push(SensorError::new(id, "Empty display name"));
        }
        if tags.is_empty() {
            errs.push(SensorError::new(id, "No SkySpark marker tags"));
        }
        for tag in tags {
            if !is_tag_name(tag) {
                errs.push(SensorError::new(
                    id,
                    format!("Invalid SkySpark marker tag '{}'", tag),
                ));
            }
        }
        if sensor.sensor_type == SensorType::Numeric {
            match unit {
                Some(unit) => {
                    if !units.contains(unit) {
                        errs.push(SensorError::new(id, "Invalid unit"));
                    }
                }
                None => (), // It's ok to have no unit, just highly uncommon.
            }
        } else {
            if unit.is_some() {
                errs.push(SensorError::new(
                    id,
                    "Has a unit but is not numeric",
                ));
            }
        }
    }

    if !all_ids_unique(&sensors) {
        errs.push(SensorError::new("-", "Some sensor ids are not unique"));
    }

    errs
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

fn clean_raw_sensor_info(sensor_info: SensorInfo) -> SensorInfo {
    let mut cleaned_tags: Vec<String> = sensor_info
        .extra_skyspark_marker_tags
        .into_iter()
        .map(|tag| tag.trim().to_owned())
        .collect();
    cleaned_tags.sort();

    SensorInfo {
        sensor_id: sensor_info.sensor_id.trim().to_owned(),
        extra_skyspark_marker_tags: cleaned_tags,
    }
}

fn clean_raw_assets(raw_assets: Vec<Asset>) -> Vec<Asset> {
    let mut assets: Vec<Asset> = raw_assets
        .into_iter()
        .map(|raw_asset| {
            let mut cleaned_tags: Vec<String> = raw_asset
                .skyspark_marker_tags
                .into_iter()
                .map(|tag| tag.trim().to_owned())
                .collect();
            cleaned_tags.sort();

            let mut cleaned_mandatory_sensors: Vec<SensorInfo> = raw_asset
                .mandatory_sensors
                .into_iter()
                .map(|sensor_info| clean_raw_sensor_info(sensor_info))
                .collect();
            cleaned_mandatory_sensors
                .sort_by(|a, b| a.sensor_id.cmp(&b.sensor_id));

            let mut cleaned_optional_sensors: Vec<SensorInfo> = raw_asset
                .optional_sensors
                .into_iter()
                .map(|sensor_info| clean_raw_sensor_info(sensor_info))
                .collect();
            cleaned_optional_sensors
                .sort_by(|a, b| a.sensor_id.cmp(&b.sensor_id));

            let mut sorted_arms_asset_type_ids = raw_asset.arms_asset_type_ids;
            sorted_arms_asset_type_ids.sort();
            sorted_arms_asset_type_ids.dedup();

            Asset {
                id: raw_asset.id.trim().to_owned(),
                display_name: raw_asset.display_name.trim().to_owned(),
                skyspark_marker_tags: cleaned_tags,
                mandatory_sensors: cleaned_mandatory_sensors,
                optional_sensors: cleaned_optional_sensors,
                arms_asset_type_ids: sorted_arms_asset_type_ids,
            }
        })
        .collect();
    assets.sort_by(|a, b| a.id.cmp(&b.id));
    assets
}

struct Args {
    assets_filepath: String,
    sensors_filepath: String,
}

fn parse_args() -> Args {
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        print_help();
    } else {
        let assets_filepath = args.remove(1);
        let sensors_filepath = args.remove(1);

        Args {
            assets_filepath,
            sensors_filepath,
        }
    }
}

fn print_help() -> ! {
    println!("Could not parse arguments.");
    println!("Example usage:");
    println!("    asset_parser '/path/to/assets.json' '/path/to/sensors.json'");
    std::process::exit(1);
}

enum Mode {
    ParseSensors {
        filepath: String,
    },
    ParseAssets {
        assets_filepath: String,
        sensors_filepath: String,
    },
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

fn to_pretty_string(json: &serde_json::Value) -> String {
    let buffer = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut serializer =
        serde_json::Serializer::with_formatter(buffer, formatter);
    json.serialize(&mut serializer).unwrap();
    String::from_utf8(serializer.into_inner()).unwrap()
}
