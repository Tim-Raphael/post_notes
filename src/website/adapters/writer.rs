use crate::website;

pub struct Writer<WebsiteBuilder: website::Build> {
    pages: WebsiteBuilder,
}

impl<B> Write for Writer<B> where B: website::Build {}
