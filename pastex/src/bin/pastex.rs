use pastex::document;
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
        .map(document::process)
        .map(|document| {
            for blk in document.outline {
                let document::Block(_, content) = blk;

                print!("<p>");
                for span in content {
                    match span {
                        document::Span::Raw(t) => print!("{}", t),
                        _ => unimplemented!(),
                    }
                }
                println!("</p>");
            }
        })
}
