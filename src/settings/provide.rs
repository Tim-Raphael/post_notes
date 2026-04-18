use std::path;

pub trait Provide {
    fn input(&self) -> &path::Path;

    fn output(&self) -> &path::Path;

    fn templates(&self) -> &path::Path;

    fn assets(&self) -> &path::Path;
}
