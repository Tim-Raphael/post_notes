mod path;

pub use path::Path;

use crate::defaults;

/// Command line arguments - mirrors [Settings] structure.
#[derive(
    Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize, clap::Parser,
)]
#[command(name = "post-notes")]
#[command(about = "Building a cute digital garden.")]
#[command(version)]
pub struct Args {
    /// Config file path.
    #[arg(short, long, default_value = defaults::DEFAULT_CONFIG_PATH )]
    #[serde(skip)]
    pub config: String,
    /// Path settings.
    #[command(flatten)]
    pub path: Path,
}
