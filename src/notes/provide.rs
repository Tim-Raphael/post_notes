use crate::notes::types;

pub trait Provide {
    fn notes(&self) -> &[types::Note];
}
