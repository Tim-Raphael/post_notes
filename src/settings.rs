use anyhow::Error;
use clap::Parser;
use config::{Config, File, FileFormat, FileSourceFile};
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::{collections::HashSet, path::PathBuf};

const CONFIG_PATH: &str = "./Config.toml";

const DEFAULT_INPUT_PATH: &str = "./notes";
const DEFAULT_OUTPUT_PATH: &str = "./output";
const DEFAULT_VOLATILE_PATH: &str = "./.temp";
const DEFAULT_TEMPLATE_PATH: &str = "./templates";
const DEFAULT_ASSET_PATH: &str = "./assets";

/// Represents the type of value the front matter field holds.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ValueType {
    /// i64
    Integer,
    /// f64
    Float,
    /// bool
    Boolean,
    /// String
    String,
    /// Vec<[ValueType]>
    Array(Box<ValueType>),
}

/// Represents a front matter field holding data of a certain [ValueType].
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
pub struct Schema(HashSet<Field>);

impl Default for Schema {
    fn default() -> Self {
        let mut raw_schema = HashSet::new();

        raw_schema.insert(Field {
            name: "title".to_string(),
            value_type: ValueType::String,
            required: true,
        });

        raw_schema.insert(Field {
            name: "description".to_string(),
            value_type: ValueType::String,
            required: true,
        });

        raw_schema.insert(Field {
            name: "tags".to_string(),
            value_type: ValueType::Array(Box::new(ValueType::String)),
            required: true,
        });

        Self(raw_schema)
    }
}

/// A wrapper around [Schema], that denotes the origin of the schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchemaValue {
    /// The user explicitly set no schema to validate against.
    None,
    /// The user did not configure any schema.
    Default(Schema),
    /// The user explicitly configured a schema to validate against.
    #[serde(untagged)]
    Custom(Schema),
}

impl Default for SchemaValue {
    fn default() -> Self {
        Self::Default(Schema::default())
    }
}

/// All settings that can be configured regarding the parsing of the
/// front matter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FrontMatterSettings {
    /// Contains information about the structure.
    pub schema: SchemaValue,
    ///An alias for the public field.
    ///
    /// This field denotes if a file should be parsed and rendered as html.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_field_alias: Option<String>,
}

/// Optional front matter settings used to parse command line arguments -
/// similar to [FrontMatterSettings].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Parser)]
struct CliFrontMatterSettings {
    ///An alias for the public field.
    #[arg(short, long)]
    pub public_field_alias: Option<String>,
}

/// All settings that can be cofnigured regarding the directories which will be
/// referenced during the site generation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PathSettings {
    /// Input directory path.
    pub input: PathBuf,
    /// Output directory path.
    pub output: PathBuf,
    // This name was choosen to allow for unique short cli argugments.
    /// Volatile (Temporary) directory path.
    pub volatile: PathBuf,
    /// Template directory path.
    pub template: PathBuf,
    /// Asset directory path.
    pub asset: PathBuf,
}

impl Default for PathSettings {
    fn default() -> Self {
        PathSettings {
            input: PathBuf::from(DEFAULT_INPUT_PATH),
            output: PathBuf::from(DEFAULT_OUTPUT_PATH),
            volatile: PathBuf::from(DEFAULT_VOLATILE_PATH),
            template: PathBuf::from(DEFAULT_TEMPLATE_PATH),
            asset: PathBuf::from(DEFAULT_ASSET_PATH),
        }
    }
}

/// Optional path settings used to parse command line arguments - mirros
/// [PathSettings].
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default, Parser,
)]
struct CliPathSettings {
    /// Input directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<PathBuf>,
    /// Output directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<PathBuf>,
    /// Volatile (Temporary) directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volatile: Option<PathBuf>,
    /// Template directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<PathBuf>,
    /// Asset directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<PathBuf>,
}

/// A single step in the build pipeline.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PipelineStep {
    /// Denotes whether to execute the step in the pipeline or not.
    pub enabled: bool,
    /// A set of path's to binaries executed before the current pipeline step is
    /// executed.
    pub pre: Option<Vec<PathBuf>>,
    /// A set of path's to binaries executed after the current pipeline step is
    /// executed.
    pub post: Option<Vec<PathBuf>>,
}

impl Default for PipelineStep {
    fn default() -> Self {
        Self {
            enabled: true,
            pre: None,
            post: None,
        }
    }
}

/// Contains settings related to the build pipeline.
///
/// Each step of the build pipeline being:
/// - **Parse**: Reading the input files and converting them into a
///   structured data format.
/// - **Bundling**: Collecting all static files and copying them into the
///   output folder.
/// - **Building**: Take the structured data and render them into HTML
///   files, which are placed in the output folder.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub struct PipelineSettings {
    /// Parse the input files.
    pub parse: PipelineStep,
    /// Bundle static assets into the output folder.
    pub bundling: PipelineStep,
    /// Building the HTML files and place them into the output folder.
    pub building: PipelineStep,
}

/// Configurable application settings which get derived from command line
/// arguments and the `Config.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Settings {
    /// Settings related to the front matter structure.
    pub front_matter: FrontMatterSettings,
    /// Settings related to the paths of input files or assets and the like.
    pub path: PathSettings,
    /// Settings related to the build pipeline.
    pub pipeline: PipelineSettings,
}

/// Command line arguments - mirrors [Settings] structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Parser)]
#[command(name = "post_notes")]
#[command(about = "Building a cute digital garden.")]
#[command(version)]
struct Args {
    /// Config file path.
    #[arg(short, long, default_value = CONFIG_PATH)]
    #[serde(skip)]
    config: String,
    /// Front matter settings.
    #[command(flatten)]
    front_matter: CliFrontMatterSettings,
    /// Path settings.
    #[command(flatten)]
    path: CliPathSettings,
}

/// Loads the configured settings from either `Config.toml` or the command line
/// arguments.
/// - If both are set the command line arguments overwrites the settings from
///   the `Config.toml`.
/// - If neither are set the default settings are used.
pub fn get_settings() -> Settings {
    let args = Args::parse();

    match Config::try_from(&Settings::default()) {
        Ok(config_default) => {
            let config_file = File::with_name(&args.config);
            let config_args = match Config::try_from(&args) {
                Ok(config) => Some(config),
                Err(err) => {
                    log::error!("Could not interpret cli arguments: {err}");
                    None
                }
            };
            match merge_settings(config_default, Some(config_file), config_args) {
                Ok(settings) => return settings,
                Err(err) => {
                    log::error!("Could not merge settings: {err}");
                }
            }
        }
        Err(err) => {
            log::error!("Could not interpret the default settings as config: {err}");
        }
    };

    log::info!(
        "Could not load settings from config file or command line arguments, using default settings instead."
    );

    Settings::default()
}

/// Read Settings from `Config.toml` or command line arguments.
fn merge_settings(
    default: Config,
    file: Option<File<FileSourceFile, FileFormat>>,
    args: Option<Config>,
) -> Result<Settings, Error> {
    let mut raw_settings = Config::builder().add_source(default);

    if let Some(file) = file {
        raw_settings = raw_settings.add_source(file.required(false));
    }

    if let Some(args) = args {
        raw_settings = raw_settings.add_source(args);
    };

    Ok(raw_settings.build()?.try_deserialize::<Settings>()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_merge_with_config() {
        let expected = Settings {
            front_matter: FrontMatterSettings {
                schema: SchemaValue::Custom(Schema(HashSet::from([Field {
                    name: "field".to_string(),
                    value_type: ValueType::String,
                    required: false,
                }]))),
                public_field_alias: None,
            },
            path: PathSettings {
                input: PathBuf::from("../notes"),
                output: DEFAULT_OUTPUT_PATH.into(),
                asset: DEFAULT_ASSET_PATH.into(),
                volatile: DEFAULT_VOLATILE_PATH.into(),
                template: DEFAULT_TEMPLATE_PATH.into(),
            },
            pipeline: PipelineSettings::default(),
        };

        let default_settings = Config::try_from(&Settings::default()).unwrap();
        let config_file = File::with_name("./tests/Config.toml");

        let produced = merge_settings(default_settings, Some(config_file), None).unwrap();

        assert_eq!(expected, produced);
    }

    #[test]
    fn test_merge_with_args() {
        let expected = Settings {
            front_matter: FrontMatterSettings::default(),
            path: PathSettings {
                input: PathBuf::from("../notes"),
                output: DEFAULT_OUTPUT_PATH.into(),
                asset: DEFAULT_ASSET_PATH.into(),
                volatile: DEFAULT_VOLATILE_PATH.into(),
                template: DEFAULT_TEMPLATE_PATH.into(),
            },
            pipeline: PipelineSettings::default(),
        };

        let default_settings = Config::try_from(&Settings::default()).unwrap();
        let args = Args::try_parse_from(["post_notes", "-i", "../notes"]).unwrap();
        let config_args = Config::try_from(&args).unwrap();

        let produced = merge_settings(default_settings, None, Some(config_args)).unwrap();

        assert_eq!(expected, produced);
    }
}
