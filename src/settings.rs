use anyhow::Error;
use clap::Parser;
use config::{Config, File};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use std::{collections::HashSet, path::PathBuf};

const CONFIG_PATH: &str = "./config.toml";

const DEFAULT_INPUT_PATH: &str = "./notes";
const DEFAULT_OUTPUT_PATH: &str = "./post_notes";
const DEFAULT_TEMP_PATH: &str = "./.temp";
const DEFAULT_TEMPLATE_PATH: &str = "./template";
const DEFAULT_ASSET_PATH: &str = "./asset";

/// Represents the type of value the front matter field holds.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
enum ValueType {
    Integer,
    Float,
    Boolean,
    String,
    Array(Box<ValueType>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
struct Field {
    /// The name of the field.
    pub name: String,
    /// Denotes the expected value type of the field.
    pub value_type: ValueType,
    /// Denotes if the field is required.
    pub required: bool,
}

/// Represents the schema of the front matter.
///
/// Each [Field] has to be unique and consists of a name, value type, and a
/// required field that denotes if the field has to be present. The only field
/// that **can't** be configured is the required field, which is a reserved
/// field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Schema(HashSet<Field>);

impl Default for Schema {
    fn default() -> Self {
        let mut raw_schema = HashSet::new();
        raw_schema.insert(Field {
            name: "title".to_string(),
            value_type: ValueType::String,
            required: true,
        });
        Self(raw_schema)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct FrontMatterSettings {
    pub schema: Schema,
    pub public_field_alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
struct PathSettings {
    pub input: PathBuf,
    pub output: PathBuf,
    pub temp: PathBuf,
    pub template: PathBuf,
    pub asset: PathBuf,
}

impl Default for PathSettings {
    fn default() -> Self {
        PathSettings {
            input: PathBuf::from(DEFAULT_INPUT_PATH),
            output: PathBuf::from(DEFAULT_OUTPUT_PATH),
            temp: PathBuf::from(DEFAULT_TEMP_PATH),
            template: PathBuf::from(DEFAULT_TEMPLATE_PATH),
            asset: PathBuf::from(DEFAULT_ASSET_PATH),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct Settings {
    pub front_matter: FrontMatterSettings,
    pub path: PathSettings,
}

pub fn load_settings() -> Settings {
    read_settings().unwrap_or_else(|err| {
        log::warn!("Could not read settings: {err}");
        log::info!("Using default settings.");
        return Settings::default();
    })
}

fn read_settings() -> Result<Settings, Error> {
    let raw_settings = Config::builder()
        .add_source(File::with_name(CONFIG_PATH))
        .build()?;

    Ok(raw_settings.try_deserialize::<Settings>()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO(Tim-Raphael): impl tests
}
