use pastex::engine;
use std::io::{self, Read};

fn main() -> anyhow::Result<()> {
    let buffer = {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    pastex_parser::parse(&buffer)
        .map_err(|err| anyhow::format_err!("Parser error: {:?}", err))
        .map(engine::process)
}
