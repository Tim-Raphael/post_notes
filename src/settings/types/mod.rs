pub mod args;

pub use args::Args;

use std::path;

use crate::defaults;

/// All settings that can be cofnigured regarding the directories which will be
/// referenced during the site generation.
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

impl Default for Settings {
    fn default() -> Self {
        Self {
            input: path::PathBuf::from(defaults::DEFAULT_INPUT_PATH),
            output: path::PathBuf::from(defaults::DEFAULT_OUTPUT_PATH),
            templates: path::PathBuf::from(defaults::DEFAULT_TEMPLATE_PATH),
            assets: path::PathBuf::from(defaults::DEFAULT_ASSET_PATH),
        }
    }
}
