use crate::{
    document::{
        metadata::{Field, Metadata},
        BlockFormat, Span,
    },
    engine::RootSpan,
};
use log::warn;
use once_cell::sync::Lazy;
use std::collections::HashMap;

type CommandName<'a> = (&'a str, Option<&'a str>);

mod inline;
mod toplevel;

macro_rules! meta_impl {
    ($name:ident) => {
        Box::new(toplevel::meta(
            stringify!(name),
            |m| &m.$name,
            |m, v| m.$name = Field::from(&v),
        ))
    };
}

macro_rules! commands_impl {
    ($hm:ident, $name:expr => $f:expr, $($r:tt)*) => {
        log::debug!("Registering command {}", $name);
        $hm.insert(($name, None), Box::new($f));
        commands_impl!($hm, $($r)*);
    };
    ($hm:ident, $ns:expr, $name:expr => $f:expr, $($r:tt)*) => {
        log::debug!("Registering command {}:{}", $ns, $name);
        $hm.insert(($name, Some($ns)), Box::new($f));
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

commands!(COMMANDS of inline::Command {
    "code" => inline::code,
    "strong" => inline::strong,
});

commands!(TOPLEVEL_COMMANDS of toplevel::Command {
    "code" => toplevel::code,
    "head1" => toplevel::header::<1>,
    "head2" => toplevel::header::<2>,
    "head3" => toplevel::header::<3>,
    "abstract" => toplevel::r#abstract,
    "meta", "title" => meta_impl!(title),
    "meta", "author" => meta_impl!(author),
    "meta", "date" => meta_impl!(date),
    "meta", "tags" => meta_impl!(keywords),
    "meta", "draft" => meta_impl!(draft),
});

pub fn toplevel_run(metadata: &mut Metadata, cmd: pastex_parser::Command) -> Vec<RootSpan> {
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

pub fn run(cmd: pastex_parser::Command) -> Vec<Span> {
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
