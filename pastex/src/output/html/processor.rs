use super::{tags, AnyTag, HtmlDocument, Tag};
use crate::document::{metadata::Metadata, Block, BlockFormat, Document, Span, SpanFormat};
use std::fmt;

#[inline]
fn r#box<T: 'static + fmt::Display>(t: T) -> AnyTag {
    Box::new(t)
}

macro_rules! attr {
    ($v:ident, $name:ident = $value:expr) => {
        $v.push((stringify!($name).to_owned(), $value.to_owned()));
    };
    ($v:ident, $name:ident = $value:expr , $($r:tt)*) => {
        $v.push((stringify!($name).to_owned(), $value.to_owned()));
        attr!($v, $($r)*);
    }
}

macro_rules! attrs {
    ($($r:tt)*) => {
        {
            let mut v = Vec::new();
            attr!(v, $($r)*);
            v
        }
    };
}

macro_rules! tag {
    (box $($r:tt)*) => {
        r#box(tag!($($r)*))
    };
    ($tag:ident) => {
        tag!($tag => Vec::new())
    };
    ($tag:ident => vec $content:expr) => {
        tag!($tag => ($content).into_iter().collect::<Vec<AnyTag>>())
    };
    ($tag:ident => $content:expr) => {
        Tag::<tags::$tag> {
            content: $content,
            .. Default::default()
        }
    };
    ($tag:ident($($r:tt)*) => $content:expr) => {
        Tag::<tags::$tag> {
            content: $content,
            attributes: attrs!($($r)*),
            .. Default::default()
        }
    };
}

fn span(s: Span) -> AnyTag {
    match s {
        Span::Text(t) => Box::new(t),
        Span::Format(f, t) => {
            let inner = t.into_iter().map(span).collect::<Vec<_>>();

            match f {
                SpanFormat::Code => tag!(box code => inner),
                SpanFormat::Strong => tag!(box strong => inner),
            }
        }
        Span::LineBreak => Box::new(tag!(br)),
    }
}

fn heading(level: usize, inner: Vec<AnyTag>) -> AnyTag {
    match level {
        1 => Box::new(tag!(h2 => inner)),
        2 => Box::new(tag!(h3 => inner)),
        3 => Box::new(tag!(h4 => inner)),
        _ => unimplemented!(),
    }
}

fn block(block: Block) -> AnyTag {
    let Block(format, content) = block;
    let inner = content.into_iter().map(span).collect::<Vec<_>>();

    match format {
        BlockFormat::Paragraph => Box::new(tag!(p => inner)),
        BlockFormat::Code => {
            tag!(box pre => vec [tag!(box code(class = "code-block") => inner)])
        }
        BlockFormat::Heading(lvl) => heading(lvl, inner),
    }
}

fn head(metadata: Metadata) -> Tag<tags::head> {
    let mut res = Vec::new();

    if let Some(value) = metadata.title {
        res.push(tag!(box title => vec [r#box(value)]));
    }

    tag!(head => res)
}

fn body(outline: Vec<Block>) -> Tag<tags::body> {
    let inner = outline.into_iter().map(block).collect::<Vec<_>>();

    tag!(body => inner)
}

pub fn output(document: Document) -> HtmlDocument {
    HtmlDocument(tag!(html => vec [
        r#box(head(document.metadata)),
        r#box(body(document.outline))
    ]))
}
