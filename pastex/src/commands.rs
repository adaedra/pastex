use crate::{
    document::{BlockFormat, Metadata, MetadataField, Span, SpanFormat},
    engine::{element, RootSpan},
};
use log::warn;
use once_cell::sync::Lazy;
use pastex_parser::{Element, Stream};
use std::collections::HashMap;

pub type CommandName<'a> = (&'a str, Option<&'a str>);
type Command = Box<dyn Fn(Stream, bool) -> Vec<Span> + Send + Sync>;
type ToplevelCommand = Box<dyn Fn(&mut Metadata, Stream, bool) -> Vec<RootSpan> + Send + Sync>;

fn code(content: Stream, _: bool) -> Vec<Span> {
    let inner = content
        .into_iter()
        .map(element)
        .flatten()
        .collect::<Vec<_>>();

    vec![Span::Format(SpanFormat::Code, inner)]
}

fn code_tl(_: &mut Metadata, content: Stream, block: bool) -> Vec<RootSpan> {
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

fn meta<T, G, S>(
    metadata: &mut Metadata,
    name: &'static str,
    get: G,
    set: S,
    content: Stream,
    _: bool,
) -> Vec<RootSpan>
where
    T: MetadataField,
    G: Fn(&Metadata) -> &T,
    S: Fn(&mut Metadata, String),
{
    if get(metadata).is_set() {
        warn!("Replacing existing metadata for {}", name);
    }
    let content = content
        .into_iter()
        .map(|el| match el {
            Element::Raw(t) => t,
            _ => panic!("oops"),
        })
        .collect::<String>();
    set(metadata, content);

    vec![]
}

fn meta_impl<T, G, S>(
    name: &'static str,
    get: G,
    set: S,
) -> impl Fn(&mut Metadata, Stream, bool) -> Vec<RootSpan>
where
    T: MetadataField,
    G: Fn(&Metadata) -> &T + Copy,
    S: Fn(&mut Metadata, String) + Copy,
{
    move |metadata, content, block| meta(metadata, name, get, set, content, block)
}

macro_rules! meta_impl {
    ($name:ident) => {
        Box::new(meta_impl(
            stringify!(name),
            |m| &m.$name,
            |m, v| m.$name = MetadataField::from(&v),
        ))
    };
}

macro_rules! commands_impl {
    ($hm:ident, $name:ident => $f:expr, $($r:tt)*) => {
        log::debug!("Registering command {}", stringify!($name));
        $hm.insert((stringify!($name), None), Box::new($f));
        commands_impl!($hm, $($r)*);
    };
    ($hm:ident, $ns:ident, $name:ident => $f:expr, $($r:tt)*) => {
        log::debug!("Registering command {}:{}", stringify!($ns), stringify!($name));
        $hm.insert((stringify!($name), Some(stringify!($ns))), Box::new($f));
        commands_impl!($hm, $($r)*);
    };
    ($hm:ident,) => {};
}

macro_rules! commands {
    ($hive:ident of $type:ty { $($r:tt)* }) => {
        static $hive: Lazy<HashMap<CommandName<'static>, $type>> = Lazy::new(|| {
            let mut hm = HashMap::<_, $type>::new();
            commands_impl!(hm, $($r)*);
            hm
        });
    };
}

commands!(COMMANDS of Command {
    code => code,
    strong => strong,
});

commands!(TOPLEVEL_COMMANDS of ToplevelCommand {
    code => code_tl,
    meta, title => meta_impl!(title),
    meta, author => meta_impl!(author),
    meta, date => meta_impl!(date),
    meta, tags => meta_impl!(keywords),
    meta, draft => meta_impl!(draft),
});

pub(crate) fn toplevel_run(metadata: &mut Metadata, cmd: pastex_parser::Command) -> Vec<RootSpan> {
    let name = (cmd.name, cmd.namespace);

    if let Some(c) = TOPLEVEL_COMMANDS.get(&name) {
        c(metadata, cmd.content, cmd.block)
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
