use super::{tags, Element, ElementBox, HtmlDocument, Tag};
use crate::document::{metadata::Metadata, Block, BlockFormat, Document, Span, SpanFormat};

trait IntoElementBox {
    fn into_element_box(self) -> ElementBox;
}

impl<T: 'static + super::tags::Tag> IntoElementBox for Tag<T> {
    fn into_element_box(self) -> ElementBox {
        Box::new(self)
    }
}

impl<T: 'static + Element> IntoElementBox for Box<T> {
    fn into_element_box(self) -> ElementBox {
        self
    }
}

impl IntoElementBox for String {
    fn into_element_box(self) -> ElementBox {
        Box::new(super::Text(self))
    }
}

impl<T: 'static + IntoElementBox> IntoElementBox for Option<T> {
    fn into_element_box(self) -> ElementBox {
        self.map(IntoElementBox::into_element_box)
            .unwrap_or_else(|| Box::new(super::Empty))
    }
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
        tag!($($r)*).into_element_box()
    };
    ($tag:ident) => {
        Tag::<tags::$tag>::default()
    };
    ($tag:ident { $($t:expr ;)* }) => {
        Tag::<tags::$tag> {
            content: [$($t.into_element_box()),*].into_iter().collect::<Vec<_>>(),
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
        Span::Text(t) => t.into_element_box(),
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
    tag!(head {
        tag!(meta(charset = "utf-8"));
        metadata.title.map(|value| tag!(title { value; }));
    })
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
