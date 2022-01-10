use pastex::document::{self, BlockFormat, Span, SpanFormat};
use std::io::{self, Read};

fn tag(name: &'static str, content: Vec<Span>) {
    print!("<{}>", name);
    print(content);
    print!("</{}>", name);
}

fn print(content: Vec<Span>) {
    for span in content {
        match span {
            document::Span::Text(t) => print!("{}", t),
            document::Span::Format(f, inner) => match f {
                SpanFormat::Code => tag("code", inner),
                SpanFormat::Strong => tag("strong", inner),
            },
            document::Span::LineBreak => print!("<br />"),
        }
    }
}

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
                let document::Block(format, content) = blk;

                match format {
                    BlockFormat::Paragraph => tag("p", content),
                    BlockFormat::Code => tag("pre", vec![Span::Format(SpanFormat::Code, content)]),
                }
                println!();
            }
        })
}
