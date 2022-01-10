use crate::{
    document::{Span, SpanFormat},
    engine::element,
};
use pastex_parser::Stream;

pub type Command = Box<dyn Fn(Stream, bool) -> Vec<Span> + Send + Sync>;

pub fn code(content: Stream, _: bool) -> Vec<Span> {
    let inner = content
        .into_iter()
        .map(element)
        .flatten()
        .collect::<Vec<_>>();

    vec![Span::Format(SpanFormat::Code, inner)]
}

pub fn strong(content: Stream, _: bool) -> Vec<Span> {
    let inner = content
        .into_iter()
        .map(element)
        .flatten()
        .collect::<Vec<_>>();

    vec![Span::Format(SpanFormat::Strong, inner)]
}
