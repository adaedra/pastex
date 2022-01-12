use super::{tags, ElementBox, HtmlDocument, Tag, Text};
use crate::document::{metadata::Metadata, Block, BlockFormat, Document, Span, SpanFormat};

#[inline]
fn r#box<T: 'static + super::Element>(t: T) -> ElementBox {
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
        Tag::<tags::$tag>::default()
    };
    ($tag:ident { $($t:expr ;)* }) => {
        Tag::<tags::$tag> {
            content: [$(r#box($t)),*].into_iter().collect::<Vec<_>>(),
            .. Default::default()
        }
    };
    ($tag:ident => $content:expr) => {
        Tag::<tags::$tag> {
            content: $content,
            .. Default::default()
        }
    };
    ($tag:ident($($r:tt)*)) => {
        Tag::<tags::$tag> {
            attributes: attrs!($($r)*),
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

fn span(s: Span) -> ElementBox {
    match s {
        Span::Text(t) => r#box(Text(t)),
        Span::Format(f, t) => {
            let inner = t.into_iter().map(span).collect::<Vec<_>>();

            match f {
                SpanFormat::Code => tag!(box code => inner),
                SpanFormat::Strong => tag!(box strong => inner),
            }
        }
        Span::LineBreak => tag!(box br),
    }
}

fn heading(level: usize, inner: Vec<ElementBox>) -> ElementBox {
    match level {
        1 => tag!(box h2 => inner),
        2 => tag!(box h3 => inner),
        3 => tag!(box h4 => inner),
        _ => unimplemented!(),
    }
}

fn block(block: Block) -> ElementBox {
    let Block(format, content) = block;
    let inner = content.into_iter().map(span).collect::<Vec<_>>();

    match format {
        BlockFormat::Paragraph => tag!(box p => inner),
        BlockFormat::Code => {
            tag!(box pre {
                tag!(code(class = "code-block") => inner);
            })
        }
        BlockFormat::Heading(lvl) => heading(lvl, inner),
    }
}

fn head(metadata: Metadata) -> Tag<tags::head> {
    let mut inner = vec![tag!(box meta(charset = "utf-8"))];

    if let Some(value) = metadata.title {
        inner.push(tag!(box title { Text(value); }));
    }

    tag!(head => inner)
}

fn body(outline: Vec<Block>) -> Tag<tags::body> {
    let inner = outline.into_iter().map(block).collect::<Vec<_>>();

    tag!(body => inner)
}

pub fn output(document: Document) -> HtmlDocument {
    HtmlDocument(tag!(html {
        head(document.metadata);
        body(document.outline);
    }))
}
