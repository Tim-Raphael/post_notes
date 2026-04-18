use std::path;

#[derive(Clone, Debug)]
pub struct Image {
    pub inner: path::PathBuf,
}
