use crate::document::{metadata::Metadata, Block, BlockFormat, Document, Span, SpanFormat};
use dolmen::{prelude::*, Fragment, RawFragment};
use dolmen_dsl::element as tag;
use std::iter::once;

fn span(s: &Span) -> Box<dyn Node> {
    match s {
        Span::Text(t) => t.into_node(),
        Span::Format(f, t) => {
            let inner = Fragment::new(t.iter().map(span));

            match f {
                SpanFormat::Code => tag!(code {{ inner }}),
                SpanFormat::Strong => tag!(strong {{ inner }}),
                SpanFormat::Link { to, blank } if *blank => {
                    tag!(a[href: {to.clone()}, target: "_blank", rel: "noopener noreferrer"] {{ inner }})
                }
                SpanFormat::Link { to, .. } => tag!(a[href: {to.clone()}] {{ inner }}),
            }
            .into_node()
        }
        Span::LineBreak => tag!(br).into_node(),
        Span::Raw(r) => unsafe { RawFragment::new(r) }.into_node(),
    }
}

fn heading(level: usize, inner: Fragment) -> Box<dyn Node> {
    match level {
        1 => tag!(h2 {{ inner }}),
        2 => tag!(h3 {{ inner }}),
        3 => tag!(h4 {{ inner }}),
        _ => unimplemented!(),
    }
    .into_node()
}

fn block(block: &Block) -> Box<dyn Node> {
    let Block(format, content) = block;
    let inner = Fragment::new(content.iter().map(span));

    match format {
        &BlockFormat::Paragraph => tag!(p {{ inner }}).into_node(),
        &BlockFormat::Code => tag!(pre {
            code[class: "code-block"] {{ inner }}
        })
        .into_node(),
        &BlockFormat::Heading(lvl) => heading(lvl, inner),
        &BlockFormat::Raw => inner.into_node(),
    }
}

fn head(metadata: &Metadata) -> Fragment {
    Fragment::new([
        tag!(meta[charset: "utf-8"]).into_node(),
        metadata
            .title
            .as_ref()
            .map(|value| tag!(title {{ value }}).into_node())
            .unwrap_or_else(|| Fragment::empty().into_node()),
    ])
}

pub fn output_fragment(fragment: &[Block]) -> Fragment {
    Fragment::new(fragment.into_iter().map(block))
}

pub fn output(document: &Document) -> (Fragment, Option<Fragment>) {
    (
        output_fragment(&document.outline),
        document
            .metadata
            .r#abstract
            .as_ref()
            .map(|blocks| output_fragment(blocks)),
    )
}

pub fn output_document(document: &Document) -> Fragment {
    let html = tag!(html[lang: "en"] {
        head {{ head(&document.metadata) }};
        body {{ output_fragment(&document.outline) }}
    })
    .into_node();
    Fragment::new(once(html))
}
