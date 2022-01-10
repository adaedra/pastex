use crate::{
    document::{BlockFormat, Span, SpanFormat},
    engine::{element, RootSpan},
};
use log::warn;
use once_cell::sync::Lazy;
use pastex_parser::Stream;
use std::collections::HashMap;

pub type CommandName<'a> = (&'a str, Option<&'a str>);
type Command = dyn Fn(Stream, bool) -> Vec<Span> + Send + Sync;
type ToplevelCommand = dyn Fn(Stream, bool) -> Vec<RootSpan> + Send + Sync;

fn code(content: Stream, _: bool) -> Vec<Span> {
    let inner = content
        .into_iter()
        .map(element)
        .flatten()
        .collect::<Vec<_>>();

    vec![Span::Format(SpanFormat::Code, inner)]
}

fn code_tl(content: Stream, block: bool) -> Vec<RootSpan> {
    if block {
        vec![RootSpan::Block(
            BlockFormat::Code,
            vec![Span::Text("[[unimplemented code block]]".to_owned())],
        )]
    } else {
        let inner = content
            .into_iter()
            .map(element)
            .flatten()
            .collect::<Vec<_>>();

        vec![RootSpan::Format(SpanFormat::Code, inner)]
    }
}

fn strong(content: Stream, _: bool) -> Vec<Span> {
    let inner = content
        .into_iter()
        .map(element)
        .flatten()
        .collect::<Vec<_>>();

    vec![Span::Format(SpanFormat::Strong, inner)]
}

static COMMANDS: Lazy<HashMap<CommandName<'static>, &Command>> = Lazy::new(|| {
    let mut hm = HashMap::<_, &Command>::new();

    hm.insert(("code", None), &code);
    hm.insert(("strong", None), &strong);

    hm
});

static TOPLEVEL_COMMANDS: Lazy<HashMap<CommandName<'static>, &ToplevelCommand>> = Lazy::new(|| {
    let mut hm = HashMap::<_, &ToplevelCommand>::new();

    hm.insert(("code", None), &code_tl);

    hm
});

pub(crate) fn toplevel_run(cmd: pastex_parser::Command) -> Vec<RootSpan> {
    let name = (cmd.name, cmd.namespace);

    if let Some(c) = TOPLEVEL_COMMANDS.get(&name) {
        c(cmd.content, cmd.block)
    } else if let Some(c) = COMMANDS.get(&name) {
        c(cmd.content, cmd.block)
            .into_iter()
            .map(Into::into)
            .collect()
    } else {
        warn!("Unknown command: {}", cmd.command_name());

        let span = Span::Text(format!("[[unknown command {}]]", cmd.command_name()));
        if cmd.block {
            vec![RootSpan::Block(BlockFormat::Paragraph, vec![span])]
        } else {
            vec![span.into()]
        }
    }
}

pub(crate) fn run(cmd: pastex_parser::Command) -> Vec<Span> {
    let name = (cmd.name, cmd.namespace);

    if let Some(c) = COMMANDS.get(&name) {
        c(cmd.content, cmd.block)
    } else {
        warn!("Unknown command: {}", cmd.command_name());
        vec![Span::Text(format!(
            "[[unknown command {}]]",
            cmd.command_name()
        ))]
    }
}
