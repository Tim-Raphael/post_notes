use anyhow::Error;
use clap::Parser;
use config::{Config, File};
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::path::PathBuf;

const CONFIG_PATH: &str = "./Config.toml";

const DEFAULT_INPUT_PATH: &str = "./notes";
const DEFAULT_OUTPUT_PATH: &str = "./output";
const DEFAULT_TEMPLATE_PATH: &str = "./templates";
const DEFAULT_ASSET_PATH: &str = "./assets";

/// All settings that can be cofnigured regarding the directories which will be
/// referenced during the site generation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PathSettings {
    /// Input directory path.
    pub input: PathBuf,
    /// Output directory path.
    pub output: PathBuf,
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
    /// Template directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<PathBuf>,
    /// Asset directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<PathBuf>,
}

/// Configurable application settings which get derived from command line
/// arguments and the `Config.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Settings {
    /// Settings related to the paths of input files or assets and the like.
    pub path: PathSettings,
}

/// Command line arguments - mirrors [Settings] structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, Parser)]
#[command(name = "post-notes")]
#[command(about = "Building a cute digital garden.")]
#[command(version)]
struct Args {
    /// Config file path.
    #[arg(short, long, default_value = CONFIG_PATH)]
    #[serde(skip)]
    config: String,
    /// Path settings.
    #[command(flatten)]
    path: CliPathSettings,
}

/// Read Settings from `Config.toml` or command line arguments.
fn merge_settings(
    default: Config,
    file: Option<Config>,
    args: Option<Config>,
) -> Result<Settings, Error> {
    let mut raw_settings = Config::builder().add_source(default);
    if let Some(file) = file {
        raw_settings = raw_settings.add_source(file);
    }
    if let Some(args) = args {
        raw_settings = raw_settings.add_source(args);
    };
    Ok(raw_settings.build()?.try_deserialize::<Settings>()?)
}

/// Loads the configured settings from either `Config.toml` or the command line
/// arguments.
/// - If both are set the command line arguments overwrites the settings from
///   the `Config.toml`.
/// - If neither are set the default settings are used.
pub fn get_settings() -> Settings {
    let args = Args::parse();
    // Interpret default settings.
    let config_default = Config::try_from(&Settings::default())
        .map_err(|err| log::error!("Could not interpret the default settings as config: {err}"))
        .ok();
    // Load and interpret config file.
    let config_file = Config::builder()
        .add_source(File::with_name(&args.config).required(false))
        .build()
        .map_err(|err| log::error!("Could not interpret config file: {err}"))
        .ok();
    // Interpret cli arguments.
    let config_args = Config::try_from(&args)
        .map_err(|err| log::error!("Could not interpret cli arguments: {err}"))
        .ok();
    // If we have a default config, try to merge everything.
    if let Some(default) = config_default {
        if let Ok(settings) = merge_settings(default, config_file, config_args) {
            return settings;
        }
        log::error!("Could not merge settings.");
    }
    log::info!(
        "Could not load settings from config file or command line arguments, using default settings instead."
    );
    Settings::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::FileFormat;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_merge_default_settings_with_config_file() {
        let expected = Settings {
            path: PathSettings {
                input: PathBuf::from("../notes"),
                output: DEFAULT_OUTPUT_PATH.into(),
                asset: DEFAULT_ASSET_PATH.into(),
                template: DEFAULT_TEMPLATE_PATH.into(),
            },
        };
        let default_settings = Config::try_from(&Settings::default()).unwrap();
        let config_file = Config::builder()
            .add_source(File::from_str("[path]\ninput='../notes'", FileFormat::Toml))
            .build()
            .unwrap();
        let produced = merge_settings(default_settings, Some(config_file), None).unwrap();
        assert_eq!(expected, produced);
    }

    #[test]
    fn test_merge_defualt_settings_with_args() {
        let expected = Settings {
            path: PathSettings {
                input: PathBuf::from("../notes"),
                output: DEFAULT_OUTPUT_PATH.into(),
                asset: DEFAULT_ASSET_PATH.into(),
                template: DEFAULT_TEMPLATE_PATH.into(),
            },
        };
        let default_settings = Config::try_from(&Settings::default()).unwrap();
        let args = Args::try_parse_from(["post_notes", "-i", "../notes"]).unwrap();
        let config_args = Config::try_from(&args).unwrap();
        let produced = merge_settings(default_settings, None, Some(config_args)).unwrap();
        assert_eq!(expected, produced);
    }
}
