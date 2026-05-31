use clap::Parser as _;
use std::path;

mod defaults {
    pub const CONFIG_PATH: &str = "./Config.toml";
    pub const INPUT_PATH: &str = "./notes";
    pub const OUTPUT_PATH: &str = "./output";
    pub const TEMPLATE_PATH: &str = "./templates";
    pub const ASSET_PATH: &str = "./assets";
}

/// All settings that can be configured regarding the directories which will be referenced during
/// the site generation.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Settings {
    /// Input directory path.
    pub input: path::PathBuf,
    /// Output directory path.
    pub output: path::PathBuf,
    /// Template directory path.
    pub templates: path::PathBuf,
    /// Asset directory paths.
    pub assets: path::PathBuf,
}

impl Settings {
    /// Loads the configured settings from either `Config.toml` or the command line arguments.
    /// - If both are set the command line arguments overwrites the settings from the `Config.toml`.
    /// - If neither are set the default settings are used.
    pub fn new() -> Self {
        let args = args::Settings::parse();

        let default = config::Config::try_from(&Self::default())
            .map_err(|err| {
                tracing::error!("Could not interpret the default settings as config: {err}")
            })
            .ok();

        let file = config::Config::builder()
            .add_source(config::File::with_name(&args.config).required(false))
            .build()
            .map_err(|err| tracing::error!("Could not interpret config file: {err}"))
            .ok();

        let args = config::Config::try_from(&args)
            .map_err(|err| tracing::error!("Could not interpret cli arguments: {err}"))
            .ok();

        if let Some(default) = default {
            let raw_settings = {
                let mut raw_settings = config::Config::builder().add_source(default);

                if let Some(file) = file {
                    raw_settings = raw_settings.add_source(file);
                }

                if let Some(args) = args {
                    raw_settings = raw_settings.add_source(args);
                };

                raw_settings
            };

            if let Ok(raw_settings) = raw_settings
                .build()
                .inspect_err(|err| tracing::error!("Could not build merged settings: {err}"))
            {
                if let Ok(settings) = raw_settings.try_deserialize::<Self>().inspect_err(|err| {
                    tracing::error!("Could not deserialize merged settings: {err}")
                }) {
                    return settings;
                }
            }
        }

        tracing::warn!(
            "Could not load settings from config file or command line arguments, using default settings instead"
        );

        Self::default()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            input: path::PathBuf::from(defaults::INPUT_PATH),
            output: path::PathBuf::from(defaults::OUTPUT_PATH),
            templates: path::PathBuf::from(defaults::TEMPLATE_PATH),
            assets: path::PathBuf::from(defaults::ASSET_PATH),
        }
    }
}

mod args {
    use super::*;

    /// Optional path settings used to parse command line arguments.
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
        clap::Parser,
        serde::Deserialize,
        serde::Serialize,
    )]
    pub(super) struct PathSettings {
        /// Input directory path.
        #[arg(short, long)]
        #[serde(skip_serializing_if = "Option::is_none")]
        input: Option<path::PathBuf>,
        /// Output directory path.
        #[arg(short, long)]
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<path::PathBuf>,
        /// Template directory path.
        #[arg(short, long)]
        #[serde(skip_serializing_if = "Option::is_none")]
        templates: Option<path::PathBuf>,
        /// Asset directory path.
        #[arg(short, long)]
        #[serde(skip_serializing_if = "Option::is_none")]
        assets: Option<path::PathBuf>,
    }

    /// Command line arguments -- mirrors [Settings] structure.
    #[derive(
        Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize, clap::Parser,
    )]
    #[command(name = "post-notes")]
    #[command(about = "Building a cute digital garden.")]
    #[command(version)]
    pub(super) struct Settings {
        /// Config file path.
        #[arg(short, long, default_value = defaults::CONFIG_PATH )]
        #[serde(skip)]
        pub config: String,
        /// Path settings.
        #[command(flatten)]
        #[serde(flatten)]
        pub path: PathSettings,
    }
}
