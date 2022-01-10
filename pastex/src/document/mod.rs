pub mod metadata;

pub use metadata::Metadata;
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

pub struct Document {
    pub outline: Vec<Block>,
    pub metadata: Metadata,
}

pub fn process(stream: Stream) -> Document {
    let mut metadata = Metadata::default();
    let outline = crate::engine::root(&mut metadata, stream);

    Document { outline, metadata }
}
