#[derive(Clone, Debug)]
pub struct Note {
    pub file_name: super::FileName,
    pub frontmatter: super::Frontmatter,
    pub body: super::Body,
}

impl Note {
    pub fn new(
        file_name: super::FileName,
        frontmatter: super::Frontmatter,
        body: super::Body,
    ) -> anyhow::Result<Self> {
        todo!()
    }
}

impl TryFrom<super::RawNote> for Note {
    type Error = anyhow::Error;

    fn try_from(value: super::RawNote) -> Result<Self, Self::Error> {
        todo!()
    }
}
