use std::ffi;

#[derive(Clone, Debug)]
pub struct FileName {
    pub inner: String,
}

impl TryFrom<ffi::OsString> for FileName {
    type Error = ffi::OsString;

    fn try_from(value: ffi::OsString) -> Result<Self, Self::Error> {
        Ok(Self {
            inner: value.into_string()?,
        })
    }
}
