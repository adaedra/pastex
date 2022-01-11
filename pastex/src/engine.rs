use crate::document::{metadata::Metadata, Block, BlockFormat, Span, SpanFormat};
use nom::Parser;
use pastex_parser::{Element, Stream};
use std::mem::take;

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

pub trait TextProcessor: Sized {
    fn process(t: &str) -> Vec<Span>;

    fn process_all(s: Stream) -> Vec<Span> {
        s.into_iter().map(element::<Self>).flatten().collect()
    }
}

pub struct InlineTextProcessor;

impl TextProcessor for InlineTextProcessor {
    fn process(t: &str) -> Vec<Span> {
        use nom::{
            bytes::complete::{take_till1, take_while1},
            combinator::value,
            multi::many0,
        };

        let whitespace = value(" ", take_while1::<_, _, ()>(char::is_whitespace));
        let text = take_till1(char::is_whitespace);

        let (_, res) = many0(text.or(whitespace)).parse(t).unwrap();
        res.into_iter().map(|t| Span::Text(t.to_owned())).collect()
    }
}

pub struct PreserveTextProcessor;

impl TextProcessor for PreserveTextProcessor {
    fn process(t: &str) -> Vec<Span> {
        use nom::{character::complete::newline, multi::many0_count};
        let (t, _) = many0_count::<_, _, (), _>(newline)(t).unwrap();

        vec![Span::Text(t.to_owned())]
    }
}

fn element<P: TextProcessor>(el: Element) -> Vec<Span> {
    match el {
        Element::Raw(text) => P::process(text),
        Element::Comment(_) => Vec::new(),
        Element::Command(cmd) => crate::commands::run(cmd),
        Element::LineBreak => vec![Span::LineBreak],
    }
}

pub fn root_spans(metadata: &mut Metadata, stream: Stream) -> Vec<RootSpan> {
    let mut text_acc = String::new();
    let mut spans = Vec::new();

    for el in stream {
        match el {
            Element::Raw(text) => {
                text_acc.push_str(text);
            }
            Element::Comment(_) => (),
            Element::Command(cmd) => {
                let res = crate::commands::toplevel_run(metadata, cmd);
                let mut res = if !res.is_empty() && !text_acc.is_empty() {
                    toplevel_text(&take(&mut text_acc))
                        .into_iter()
                        .chain(res.into_iter())
                        .collect()
                } else {
                    res
                };

                spans.append(&mut res);
            }
            Element::LineBreak => spans.push(RootSpan::LineBreak),
        }
    }

    if !text_acc.is_empty() {
        let mut res = toplevel_text(&text_acc);
        spans.append(&mut res);
    }

    spans
}

pub fn root(metadata: &mut Metadata, stream: Stream) -> Vec<Block> {
    let document = root_spans(metadata, stream);
    let mut outline = Vec::new();
    let mut para = Vec::new();

    for span in document {
        match span {
            RootSpan::Text(t) => {
                if let Some(Span::Text(ref mut prev)) = para.last_mut() {
                    prev.push_str(&t);
                } else {
                    para.push(Span::Text(t));
                }
            }
            RootSpan::Format(f, s) => para.push(Span::Format(f, s)),
            RootSpan::LineBreak => para.push(Span::LineBreak),
            RootSpan::ParagraphBreak => {
                if !para.is_empty() {
                    let para = take(&mut para);
                    outline.push(Block(BlockFormat::Paragraph, para));
                }
            }
            RootSpan::Block(f, s) => {
                if !para.is_empty() {
                    let para = take(&mut para);
                    outline.push(Block(BlockFormat::Paragraph, para));
                }

                outline.push(Block(f, s));
            }
        }
    }

    outline
}
