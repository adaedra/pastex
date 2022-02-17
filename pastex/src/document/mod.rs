pub mod metadata;

use metadata::Metadata;
use pastex_parser::Stream;

use crate::engine::TextProcessor;

#[derive(Debug)]
pub enum BlockFormat {
    Paragraph,
    Code,
    Heading(usize),
    Raw,
}

#[derive(Debug)]
pub enum SpanFormat {
    Code,
    Strong,
    Link { to: String, blank: bool },
}

#[derive(Debug)]
pub enum Span {
    Text(String),
    Format(SpanFormat, Vec<Span>),
    LineBreak,
    Raw(String),
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

pub fn process_fragment_stream(stream: Stream) -> Vec<Block> {
    vec![Block(
        BlockFormat::Paragraph,
        crate::engine::InlineTextProcessor::process_all(stream),
    )]
}

pub fn process(path: &std::path::Path) -> std::io::Result<Document> {
    let buf = std::fs::read_to_string(path)?;
    Ok(process_stream(pastex_parser::parse(&buf).unwrap()))
}

pub fn process_fragment(fragment: &str) -> Vec<Block> {
    process_fragment_stream(pastex_parser::parse(fragment).unwrap())
}
