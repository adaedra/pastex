use pastex::{document, output::html};
use std::io::{self, Read};

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let buffer = {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    pastex_parser::parse(&buffer)
        .map_err(|err| anyhow::format_err!("Parser error: {:?}", err))
        .map(document::process_stream)
        .map(|document| html::output_document(&document))
        .map(|output| println!("{}", output))
}
