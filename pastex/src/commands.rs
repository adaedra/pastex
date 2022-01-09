use crate::{
    document::{Span, SpanFormat},
    engine::{element, TopToken},
};
use log::warn;
use once_cell::sync::Lazy;
use pastex_parser::Stream;
use std::collections::HashMap;

pub type CommandName<'a> = (&'a str, Option<&'a str>);
type Command = dyn Fn(Stream) -> Vec<Span> + Send + Sync;
type BlkCommand = dyn Fn(Stream) -> Vec<TopToken> + Send + Sync;

fn code(content: Stream) -> Vec<Span> {
    let inner = content
        .into_iter()
        .map(element)
        .flatten()
        .collect::<Vec<_>>();

    vec![Span::Format(SpanFormat::Code, inner)]
}

fn code_blk(_: Stream) -> Vec<TopToken> {
    vec![TopToken::Text(vec![Span::Raw(
        "[[unimplemented code block]]".to_owned(),
    )])]
}

static COMMANDS: Lazy<HashMap<CommandName<'static>, &Command>> = Lazy::new(|| {
    let mut hm = HashMap::<_, &Command>::new();

    hm.insert(("code", None), &code);

    hm
});

static BLOCK_COMMANDS: Lazy<HashMap<CommandName<'static>, &BlkCommand>> = Lazy::new(|| {
    let mut hm = HashMap::<_, &BlkCommand>::new();

    hm.insert(("code", None), &code_blk);

    hm
});

pub(crate) fn run_blk(cmd: pastex_parser::Command) -> Vec<TopToken> {
    let name = (cmd.name, cmd.namespace);

    if let Some(c) = BLOCK_COMMANDS.get(&name) {
        c(cmd.content)
    } else {
        warn!("Unknown block command: {}", cmd.command_name());
        vec![TopToken::Text(vec![Span::Raw(format!(
            "[[unknown block command {}]]",
            cmd.command_name()
        ))])]
    }
}

pub(crate) fn run(cmd: pastex_parser::Command) -> Vec<Span> {
    let name = (cmd.name, cmd.namespace);

    if let Some(c) = COMMANDS.get(&name) {
        c(cmd.content)
    } else {
        warn!("Unknown command: {}", cmd.command_name());
        vec![Span::Raw(format!(
            "[[unknown command {}]]",
            cmd.command_name()
        ))]
    }
}
