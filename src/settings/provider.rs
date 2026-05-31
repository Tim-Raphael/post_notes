use clap::Parser as _;
use std::path;

use crate::settings;

#[derive(Clone, Debug)]
pub struct Provider {
    settings: settings::types::Settings,
}

impl Provider {
    /// Loads the configured settings from either `Config.toml` or the command line
    /// arguments.
    /// - If both are set the command line arguments overwrites the settings from
    ///   the `Config.toml`.
    /// - If neither are set the default settings are used.
    pub fn new() -> Self {
        let args = settings::types::Args::parse();

        let default = config::Config::try_from(&settings::types::Settings::default())
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
                if let Ok(settings) = raw_settings
                    .try_deserialize::<settings::types::Settings>()
                    .inspect_err(|err| {
                        tracing::error!("Could not deserialize merged settings: {err}")
                    })
                {
                    return Self { settings };
                }
            }
        }

        tracing::warn!(
            "Could not load settings from config file or command line arguments, using default settings instead"
        );

        return Self {
            settings: settings::types::Settings::default(),
        };
    }
}

impl settings::Provide for Provider {
    fn input(&self) -> &path::Path {
        &self.settings.input
    }

    fn output(&self) -> &path::Path {
        &self.settings.output
    }

    fn templates(&self) -> &path::Path {
        &self.settings.templates
    }

    fn assets(&self) -> &path::Path {
        &self.settings.assets
    }
}

//#[cfg(test)]
//mod tests {
//    use std::path::PathBuf;
//
//    use super::*;
//    use config::FileFormat;
//    use pretty_assertions::assert_eq;
//
//    #[test]
//    fn test_merge_default_settings_with_config_file() {
//        let expected = settings::types::settings::Settings {
//            path: settings::types::settings::Path {
//                input: PathBuf::from("../notes"),
//                ..settings::types::settings::Path::default()
//            },
//        };
//        let default_settings = Config::try_from(&settings::types::settings::Settings::default()).unwrap();
//        let config_file = Config::builder()
//            .add_source(File::from_str("[path]\ninput='../notes'", FileFormat::Toml))
//            .build()
//            .unwrap();
//        let produced = merge_settings(default_settings, Some(config_file), None).unwrap();
//
//        assert_eq!(expected, produced);
//    }
//
//    #[test]
//    fn test_merge_default_settings_with_args() {
//        let expected = settings::types::settings::Settings {
//            path: settings::types::settings::Path {
//                input: PathBuf::from("../notes"),
//                ..settings::types::settings::Path::default()
//            },
//        };
//        let default_settings = Config::try_from(&settings::types::settings::Settings::default()).unwrap();
//        let args =
//            settings::types::settings::cli::Args::try_parse_from(["post_notes", "-i", "../notes"]).unwrap();
//        let config_args = Config::try_from(&args).unwrap();
//        let produced = merge_settings(default_settings, None, Some(config_args)).unwrap();
//
//        assert_eq!(expected, produced);
//    }
//}
