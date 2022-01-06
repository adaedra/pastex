use once_cell::sync::Lazy;
use pastex_parser::{self as parser, Element, Stream};
use std::collections::HashMap;

fn strong(inner: &Stream) -> String {
    format!("<strong>{}</strong>", stream(inner))
}

fn code(inner: &Stream) -> String {
    format!("<code>{}</code>", stream(inner))
}

fn code_block(inner: &Stream) -> String {
    format!("<pre><code>{}</code></pre>", stream(inner))
}

fn r#abstract(inner: &Stream) -> String {
    format!(r#"<div class="abstract">{}</div>"#, stream(inner))
}

fn head(level: usize, inner: &Stream) -> String {
    format!("<h{}>{}</h{}>", level, stream(inner), level)
}

type Command = dyn Fn(&Stream) -> String;

const COMMANDS: Lazy<HashMap<&'static str, &Command>> = Lazy::new(|| {
    let mut map = HashMap::<_, &Command>::new();

    map.insert("strong", &strong);
    map.insert("code", &code);
    map.insert("head1", &|i| head(1, i));
    map.insert("head2", &|i| head(2, i));
    map.insert("head3", &|i| head(3, i));

    map
});

const BLOCK_COMMANDS: Lazy<HashMap<&'static str, &Command>> = Lazy::new(|| {
    let mut map = HashMap::<_, &Command>::new();

    map.insert("code", &code_block);
    map.insert("abstract", &r#abstract);

    map
});

fn command(cmd: &parser::Command) -> String {
    let commands = if cmd.block { BLOCK_COMMANDS } else { COMMANDS };
    if let Some(f) = commands.get(cmd.name) {
        f(&cmd.content)
    } else {
        format!("[[no such function {}]]", cmd.command_name())
    }
}

fn raw(t: &str) -> String {
    t.to_owned()
}

fn element(element: &Element) -> String {
    match element {
        &Element::Command(ref cmd) => command(cmd),
        &Element::Raw(t) => raw(t),
        &Element::Comment(_) => String::new(),
        o => unimplemented!("{:?}", o),
    }
}

fn stream(tree: &Stream) -> String {
    tree.iter().map(element).collect()
}

pub fn process(tree: Stream) {
    for el in tree.iter() {
        print!("{}", element(el));
    }
}
