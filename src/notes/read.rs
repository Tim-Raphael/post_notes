use std::io;
use tokio_stream;

use crate::notes::types;

pub trait Read {
    async fn raw_notes(&self) -> impl tokio_stream::Stream<Item = io::Result<types::RawNote>>;
}
