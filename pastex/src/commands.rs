use crate::{
    document::SpanFormat,
    engine::{element, RootSpan},
};
use log::warn;
use once_cell::sync::Lazy;
use pastex_parser::Stream;
use std::collections::HashMap;

pub type CommandName<'a> = (&'a str, Option<&'a str>);
type Command = dyn Fn(Stream) -> Vec<RootSpan> + Send + Sync;

fn code(content: Stream) -> Vec<RootSpan> {
    let inner = content
        .into_iter()
        .map(element)
        .flatten()
        .collect::<Vec<_>>();

    vec![RootSpan::Format(SpanFormat::Code, inner)]
}

fn code_blk(_: Stream) -> Vec<RootSpan> {
    vec![RootSpan::Text("[[unimplemented code block]]".to_owned())]
}

static COMMANDS: Lazy<HashMap<CommandName<'static>, &Command>> = Lazy::new(|| {
    let mut hm = HashMap::<_, &Command>::new();

    hm.insert(("code", None), &code);

    hm
});

static BLOCK_COMMANDS: Lazy<HashMap<CommandName<'static>, &Command>> = Lazy::new(|| {
    let mut hm = HashMap::<_, &Command>::new();

    hm.insert(("code", None), &code_blk);

    hm
});

pub(crate) fn run_blk(cmd: pastex_parser::Command) -> Vec<RootSpan> {
    let name = (cmd.name, cmd.namespace);

    if let Some(c) = BLOCK_COMMANDS.get(&name) {
        c(cmd.content)
    } else {
        warn!("Unknown block command: {}", cmd.command_name());
        vec![RootSpan::Text(format!(
            "[[unknown block command {}]]",
            cmd.command_name()
        ))]
    }
}

pub(crate) fn run(cmd: pastex_parser::Command) -> Vec<RootSpan> {
    let name = (cmd.name, cmd.namespace);

    if let Some(c) = COMMANDS.get(&name) {
        c(cmd.content)
    } else {
        warn!("Unknown command: {}", cmd.command_name());
        vec![RootSpan::Text(format!(
            "[[unknown command {}]]",
            cmd.command_name()
        ))]
    }
}
