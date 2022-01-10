use crate::{
    document::{metadata, BlockFormat, Metadata, Span, SpanFormat},
    engine::{element, RootSpan},
};
use log::warn;
use pastex_parser::{Element, Stream};

pub type Command = Box<dyn Fn(&mut Metadata, Stream, bool) -> Vec<RootSpan> + Send + Sync>;

pub fn code(_: &mut Metadata, content: Stream, block: bool) -> Vec<RootSpan> {
    if block {
        vec![RootSpan::Block(
            BlockFormat::Code,
            vec![Span::Text("[[unimplemented code block]]".to_owned())],
        )]
    } else {
        let inner = content
            .into_iter()
            .map(element)
            .flatten()
            .collect::<Vec<_>>();

        vec![RootSpan::Format(SpanFormat::Code, inner)]
    }
}

fn meta_impl<T, G, S>(
    metadata: &mut Metadata,
    name: &'static str,
    get: G,
    set: S,
    content: Stream,
    _: bool,
) -> Vec<RootSpan>
where
    T: metadata::Field,
    G: Fn(&Metadata) -> &T,
    S: Fn(&mut Metadata, String),
{
    if get(metadata).is_set() {
        warn!("Replacing existing metadata for {}", name);
    }
    let content = content
        .into_iter()
        .map(|el| match el {
            Element::Raw(t) => t,
            _ => panic!("oops"),
        })
        .collect::<String>();
    set(metadata, content);

    vec![]
}

pub fn meta<T, G, S>(
    name: &'static str,
    get: G,
    set: S,
) -> impl Fn(&mut Metadata, Stream, bool) -> Vec<RootSpan>
where
    T: metadata::Field,
    G: Fn(&Metadata) -> &T + Copy,
    S: Fn(&mut Metadata, String) + Copy,
{
    move |metadata, content, block| meta_impl(metadata, name, get, set, content, block)
}