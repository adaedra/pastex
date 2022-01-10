use pastex_parser::Stream;

#[derive(Debug)]
pub enum BlockFormat {
    Paragraph,
    Code,
}

#[derive(Debug)]
pub enum SpanFormat {
    Code,
    Strong,
}

#[derive(Debug)]
pub enum Span {
    Text(String),
    Format(SpanFormat, Vec<Span>),
    LineBreak,
}

#[derive(Debug)]
pub struct Block(pub BlockFormat, pub Vec<Span>);

pub trait MetadataField {
    fn is_set(&self) -> bool;
    fn from(s: &str) -> Self;
}

impl MetadataField for Option<String> {
    fn is_set(&self) -> bool {
        self.is_some()
    }

    fn from(s: &str) -> Self {
        Some(s.to_owned())
    }
}

impl MetadataField for bool {
    fn is_set(&self) -> bool {
        false
    }

    fn from(_: &str) -> Self {
        true
    }
}

impl MetadataField for Vec<String> {
    fn is_set(&self) -> bool {
        !self.is_empty()
    }

    fn from(s: &str) -> Self {
        s.split(',').map(str::trim).map(str::to_owned).collect()
    }
}

#[derive(Debug)]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub keywords: Vec<String>,
    pub draft: bool,
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            title: None,
            author: None,
            date: None,
            keywords: Vec::new(),
            draft: false,
        }
    }
}

pub struct Document {
    pub outline: Vec<Block>,
    pub metadata: Metadata,
}

pub fn process(stream: Stream) -> Document {
    let mut metadata = Metadata::default();
    let outline = crate::engine::root(&mut metadata, stream);

    Document { outline, metadata }
}
