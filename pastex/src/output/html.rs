use crate::document::{metadata::Metadata, Block, BlockFormat, Document, Span, SpanFormat};
use dolmen::{html, tag, ElementBox, Fragment, HtmlDocument, IntoElementBox, RawHTML, Tag};

fn span(s: &Span) -> ElementBox {
    match s {
        Span::Text(t) => t.into_element_box(),
        Span::Format(f, t) => {
            let inner = t.iter().map(span).collect::<Vec<_>>();

            match f {
                SpanFormat::Code => tag!(box code => inner),
                SpanFormat::Strong => tag!(box strong => inner),
                SpanFormat::Link { to, blank } if *blank => {
                    tag!(box a(href = to, target = "_blank", rel = "noopener noreferrer") => inner)
                }
                SpanFormat::Link { to, .. } => tag!(box a(href = to) => inner),
            }
        }
        Span::LineBreak => tag!(box br),
        Span::Raw(r) => Box::new(unsafe { RawHTML::from(r.clone()) }),
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

fn block(block: &Block) -> ElementBox {
    let Block(format, content) = block;
    let inner = content.iter().map(span).collect::<Vec<_>>();

    match format {
        &BlockFormat::Paragraph => tag!(box p => inner),
        &BlockFormat::Code => {
            tag!(box pre {
                tag!(code(class = "code-block") => inner);
            })
        }
        &BlockFormat::Heading(lvl) => heading(lvl, inner),
        &BlockFormat::Raw => Fragment::from(inner).into_element_box(),
    }
}

fn head(metadata: &Metadata) -> Tag<html::head> {
    tag!(head {
        tag!(meta(charset = "utf-8"));
        metadata.title.as_ref().map(|value| tag!(title { value; }));
    })
}

pub fn output_fragment(fragment: &[Block]) -> Vec<ElementBox> {
    fragment.into_iter().map(block).collect()
}

pub fn output(document: &Document) -> (Vec<ElementBox>, Option<Vec<ElementBox>>) {
    (
        output_fragment(&document.outline),
        document
            .metadata
            .r#abstract
            .as_ref()
            .map(|blocks| output_fragment(blocks)),
    )
}

pub fn output_document(document: &Document) -> HtmlDocument {
    HtmlDocument(tag!(html {
        head(&document.metadata);
        tag!(body => output_fragment(&document.outline));
    }))
}
