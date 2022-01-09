use pastex_parser::Stream;

#[derive(Debug)]
pub enum BlockFormat {
    Paragraph,
}

#[derive(Debug)]
pub enum SpanFormat {
    Code,
}

#[derive(Debug)]
pub enum Span {
    Raw(String),
    Format(SpanFormat, Vec<Span>),
}

#[derive(Debug)]
pub struct Block(pub BlockFormat, pub Vec<Span>);

pub struct Document {
    pub outline: Vec<Block>,
}

pub fn process(stream: Stream) -> Document {
    let outline = crate::engine::root(stream);

    Document { outline }
}
