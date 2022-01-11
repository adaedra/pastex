use crate::{
    document::{Span, SpanFormat},
    engine::{self, TextProcessor},
};
use pastex_parser::Stream;

pub type Command = Box<dyn Fn(Stream, bool) -> Vec<Span> + Send + Sync>;

pub fn code(content: Stream, _: bool) -> Vec<Span> {
    let inner = engine::PreserveTextProcessor::process_all(content);
    vec![Span::Format(SpanFormat::Code, inner)]
}

pub fn strong(content: Stream, _: bool) -> Vec<Span> {
    let inner = engine::InlineTextProcessor::process_all(content);
    vec![Span::Format(SpanFormat::Strong, inner)]
}
