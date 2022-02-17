use crate::{
    document::{Span, SpanFormat},
    engine::{self, TextProcessor},
};
use pastex_parser::{ParamValue, Params, Stream};

pub type Command = Box<dyn Fn(Stream, &Params, bool) -> Vec<Span> + Send + Sync>;

pub fn code(content: Stream, _: &Params, _: bool) -> Vec<Span> {
    let inner = engine::PreserveTextProcessor::process_all(content);
    vec![Span::Format(SpanFormat::Code, inner)]
}

pub fn strong(content: Stream, _: &Params, _: bool) -> Vec<Span> {
    let inner = engine::InlineTextProcessor::process_all(content);
    vec![Span::Format(SpanFormat::Strong, inner)]
}

pub fn link(content: Stream, params: &Params, _: bool) -> Vec<Span> {
    let inner = engine::InlineTextProcessor::process_all(content);
    if let Some(ParamValue::Text(to)) = params.get("to") {
        vec![Span::Format(
            SpanFormat::Link {
                to: to.to_string(),
                blank: params.contains_key("blank"),
            },
            inner,
        )]
    } else {
        panic!(r"\link without to");
    }
}

pub fn raw(content: Stream, _: &Params, _: bool) -> Vec<Span> {
    let inner = engine::PreserveTextProcessor::process_all(content);
    match inner.into_iter().next() {
        Some(Span::Text(span)) => vec![Span::Raw(span)],
        None => Vec::new(),
        _ => unreachable!(),
    }
}
