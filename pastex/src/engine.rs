use crate::document::{Block, BlockFormat, Span};
use nom::Parser;
use pastex_parser::{Element, Stream};

#[derive(Debug)]
pub enum TopToken {
    Text(Vec<Span>),
    Break,
}

fn raw(t: &str) -> Vec<TopToken> {
    use nom::{
        bytes::complete::{take_till1, take_while1},
        character::complete::char,
        combinator::{not, value},
        multi::{many1, many1_count},
    };

    // Paragraph breaks
    let pbreak = char::<_, ()>('\n')
        .and(many1_count(char('\n')))
        .map(|_| TopToken::Break);

    // Inner line breaks, but not paragraph breaks
    let linebr = value(" ", char('\n').and(not(char('\n'))));
    // Either in-line whitespace or a word
    let text_item = take_till1(char::is_whitespace).or(linebr.or(value(
        " ",
        take_while1(|c: char| c != '\n' && c.is_whitespace()),
    )));
    // Assemble the previous parsers to get whole paragraphs at once
    let text = many1(text_item)
        .map(|res| TopToken::Text(vec![Span::Raw(res.into_iter().collect::<String>())]));

    let (_, tokens) = many1(pbreak.or(text))(t).unwrap();
    tokens
        .into_iter()
        .skip_while(|t| matches!(t, TopToken::Break))
        .collect()
}

pub(crate) fn element(el: Element) -> Vec<Span> {
    match el {
        Element::Raw(text) => vec![Span::Raw(text.to_owned())],
        Element::Comment(_) => Vec::new(),
        Element::Command(_) => unimplemented!(),
        Element::LineBreak => vec![Span::Raw("\n".to_string())],
    }
}

pub(crate) fn root_element(el: Element) -> Vec<TopToken> {
    match el {
        Element::Raw(text) => raw(text),
        Element::Comment(_) => Vec::new(),
        Element::Command(cmd) => {
            if cmd.block {
                crate::commands::run_blk(cmd)
            } else {
                vec![TopToken::Text(crate::commands::run(cmd))]
            }
        }
        Element::LineBreak => vec![TopToken::Text(vec![Span::Raw("\n".to_string())])],
    }
}

fn root_tokens(stream: Stream) -> Vec<TopToken> {
    stream
        .into_iter()
        .map(root_element)
        .flatten()
        .collect::<Vec<_>>()
}

pub(crate) fn root(stream: Stream) -> Vec<Block> {
    let tokens = root_tokens(stream);
    let mut outline = Vec::new();
    let mut para = Vec::new();

    for tk in tokens {
        match tk {
            TopToken::Text(span) => {
                para.push(span);
            }
            TopToken::Break => {
                if !para.is_empty() {
                    let para = std::mem::replace(&mut para, Vec::new());
                    outline.push(Block(
                        BlockFormat::Paragraph,
                        para.into_iter().flatten().collect(),
                    ));
                }
            }
        }
    }

    outline
}
