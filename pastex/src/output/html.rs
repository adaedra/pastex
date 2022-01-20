use crate::document::{metadata::Metadata, Block, BlockFormat, Document, Span, SpanFormat};
use dolmen::{html, tag, ElementBox, HtmlDocument, IntoElementBox, Tag};

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

fn head(metadata: Metadata) -> Tag<html::head> {
    tag!(head {
        tag!(meta(charset = "utf-8"));
        metadata.title.map(|value| tag!(title { &value; }));
    })
}

fn body(outline: Vec<Block>) -> Vec<ElementBox> {
    outline.into_iter().map(block).collect::<Vec<_>>()
}

pub fn output(mut document: Document) -> (Vec<ElementBox>, Option<Vec<ElementBox>>) {
    (
        body(document.outline),
        document.metadata.r#abstract.take().map(body),
    )
}

pub fn output_document(document: Document) -> HtmlDocument {
    HtmlDocument(tag!(html {
        head(document.metadata);
        tag!(body => body(document.outline));
    }))
}
