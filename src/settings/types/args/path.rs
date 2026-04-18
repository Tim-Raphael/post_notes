use std::path;

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
pub struct Path {
    /// Input directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<path::PathBuf>,
    /// Output directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<path::PathBuf>,
    /// Template directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub templates: Option<path::PathBuf>,
    /// Asset directory path.
    #[arg(short, long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ' ')]
    pub assets: Option<Vec<path::PathBuf>>,
}
