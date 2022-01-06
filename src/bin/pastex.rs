use pastex::{engine, parser};
use std::io::{self, Read};

fn main() -> anyhow::Result<()> {
    let buffer = {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        buffer
    };

    match parser::document(&buffer) {
        Ok((_, res)) => {
            engine::process(res);
        }
        Err(e) => anyhow::bail!("Parser error: {:?}", e),
    }

    Ok(())
}
