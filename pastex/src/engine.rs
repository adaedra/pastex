use crate::document::{Block, BlockFormat, Span, SpanFormat};
use nom::Parser;
use pastex_parser::{Element, Stream};

pub enum RootSpan {
    Text(String),
    Block(BlockFormat, Vec<Span>),
    Format(SpanFormat, Vec<Span>),
    ParagraphBreak,
    LineBreak,
}

impl From<Span> for RootSpan {
    fn from(span: Span) -> Self {
        match span {
            Span::Format(f, s) => RootSpan::Format(f, s),
            Span::LineBreak => RootSpan::LineBreak,
            Span::Text(t) => RootSpan::Text(t),
        }
    }
}

fn toplevel_text(t: &str) -> Vec<RootSpan> {
    use nom::{
        bytes::complete::{take_till1, take_while1},
        character::complete::char,
        combinator::{not, value},
        multi::{many1, many1_count},
    };

    // Paragraph breaks
    let pbreak = char::<_, ()>('\n')
        .and(many1_count(char('\n')))
        .map(|_| RootSpan::ParagraphBreak);

    // Inner line breaks, but not paragraph breaks
    let linebr = value(" ", char('\n').and(not(char('\n'))));
    // Either in-line whitespace or a word
    let text_item = take_till1(char::is_whitespace).or(linebr.or(value(
        " ",
        take_while1(|c: char| c != '\n' && c.is_whitespace()),
    )));
    // Assemble the previous parsers to get whole paragraphs at once
    let text = many1(text_item).map(|res| RootSpan::Text(res.into_iter().collect::<String>()));

    let (_, tokens) = many1(pbreak.or(text))(t).unwrap();
    tokens
        .into_iter()
        .skip_while(|t| matches!(t, RootSpan::ParagraphBreak))
        .collect()
}

pub(crate) fn element(el: Element) -> Vec<Span> {
    match el {
        Element::Raw(text) => vec![Span::Text(text.to_owned())],
        Element::Comment(_) => Vec::new(),
        Element::Command(cmd) => crate::commands::run(cmd),
        Element::LineBreak => vec![Span::LineBreak],
    }
}

pub(crate) fn root_element(el: Element) -> Vec<RootSpan> {
    match el {
        Element::Raw(text) => toplevel_text(text),
        Element::Comment(_) => Vec::new(),
        Element::Command(cmd) => crate::commands::toplevel_run(cmd),
        Element::LineBreak => vec![RootSpan::LineBreak],
    }
}

fn root_spans(stream: Stream) -> Vec<RootSpan> {
    stream
        .into_iter()
        .map(root_element)
        .flatten()
        .collect::<Vec<_>>()
}

pub(crate) fn root(stream: Stream) -> Vec<Block> {
    let document = root_spans(stream);
    let mut outline = Vec::new();
    let mut para = Vec::new();

    for span in document {
        match span {
            RootSpan::Text(t) => para.push(Span::Text(t)),
            RootSpan::Format(f, s) => para.push(Span::Format(f, s)),
            RootSpan::LineBreak => para.push(Span::LineBreak),
            RootSpan::ParagraphBreak => {
                if !para.is_empty() {
                    let para = std::mem::replace(&mut para, Vec::new());
                    outline.push(Block(BlockFormat::Paragraph, para));
                }
            }
            RootSpan::Block(f, s) => {
                if !para.is_empty() {
                    let para = std::mem::replace(&mut para, Vec::new());
                    outline.push(Block(BlockFormat::Paragraph, para));
                }

                outline.push(Block(f, s));
            }
        }
    }

    outline
}
