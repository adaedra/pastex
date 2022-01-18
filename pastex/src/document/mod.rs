pub mod metadata;

use metadata::Metadata;
use pastex_parser::Stream;

#[derive(Debug)]
pub enum BlockFormat {
    Paragraph,
    Code,
    Heading(usize),
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

pub fn process_stream(stream: Stream) -> Document {
    let mut metadata = Metadata::default();
    let outline = crate::engine::root(&mut metadata, stream);

    Document { outline, metadata }
}

pub fn process(path: &std::path::Path) -> std::io::Result<Document> {
    let buf = std::fs::read_to_string(path)?;
    Ok(process_stream(pastex_parser::parse(&buf).unwrap()))
}
