use clap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use toml;

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
enum FieldValue {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(String),
    Array(Vec<FieldValue>),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
enum ValueType {
    Integer,
    Float,
    Boolean,
    String,
    Array,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
struct Field {
    /// The name of the field.
    name: String,
    /// Denotes the expected type of the field.
    value_type: ValueType,
    /// Denotes if the field is required.
    required: bool,
}

struct FrontMatterSchema(HashSet<Field>);

struct Config {
    pub front_matter: Option<Vec<String>>,
    pub input_path: Option<String>,
    pub output_path: Option<String>,
    pub temp_path: Option<String>,
    pub template_path: Option<String>,
    pub assets_path: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO(Tim-Raphael): impl tests
}
