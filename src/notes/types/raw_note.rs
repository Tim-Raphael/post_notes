use std::ffi;

#[derive(Clone, Debug)]
pub struct RawNote {
    pub file_name: ffi::OsString,
    pub content: String,
}

impl RawNote {
    pub fn new(file_name: ffi::OsString, content: String) -> Self {
        Self { file_name, content }
    }
}

impl From<(ffi::OsString, String)> for RawNote {
    fn from(value: (ffi::OsString, String)) -> Self {
        Self::new(value.0, value.1)
    }
}
