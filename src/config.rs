use clap;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf};
use toml;

const DEFAULT_FRONT_MATTER_SCHEMA: &str = r#"

"#;

const DEFAULT_PATH_INPUT: &str = "./notes";
const DEFAULT_PATH_OUTPUT: &str = "./post_notes";
const DEFAULT_PATH_TEMP: &str = "./.temp";
const DEFAULT_PATH_TEMPLATE: &str = "./template";
const DEFAULT_PATH_ASSET: &str = "./asset";

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
/// that **can't** be configured is the required field, wich is a reserved
/// field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Schema(HashSet<Field>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
struct PublicField {
    pub alias: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FrontMatterConfig {
    pub schema: Schema,
    pub public_field: PublicField,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
struct PathConfig {
    pub input: PathBuf,
    pub output: PathBuf,
    pub temp: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Config {
    pub front_matter: FrontMatterConfig,
    pub path: PathConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO(Tim-Raphael): impl tests
}
