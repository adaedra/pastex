use either::Either;
use nom::Parser;
use std::collections::HashMap;

pub type Params<'b> = HashMap<&'b str, Option<&'b str>>;
pub type Stream<'b> = Vec<Element<'b>>;

#[derive(Debug)]
pub struct Command<'b> {
    pub name: &'b str,
    pub namespace: Option<&'b str>,
    pub content: Stream<'b>,
    pub params: Params<'b>,
    pub block: bool,
}

#[derive(Debug)]
pub enum Element<'b> {
    Command(Command<'b>),
    Raw(&'b str),
    Comment(&'b str),
    LineBreak,
}

enum CommandType<'b> {
    Normal(Command<'b>),
    Start(Command<'b>),
    End(Command<'b>),
    Escape(&'b str),
}

pub type Result<'t, T> = nom::IResult<&'t str, T>;

struct Pair {
    open: char,
    close: char,
}

impl Pair {
    const fn make(open: char, close: char) -> Pair {
        Pair { open, close }
    }
}

const COMMAND_CHAR: char = '\\';
const NAMESPACE_CHAR: char = ':';
const COMMAND_CONTENT_CHARS: Pair = Pair::make('{', '}');
const COMMAND_PARAMS_CHARS: Pair = Pair::make('[', ']');
const COMMAND_PARAMS_SEP_CHAR: char = ',';
const COMMENT_CHAR: char = '%';
const LINE_BREAK_CHAR: char = '\n';
const COMMAND_BLOCK_START: &str = "begin";
const COMMAND_BLOCK_END: &str = "end";

fn ident(cur: &str) -> Result<&str> {
    use nom::bytes::complete::take_while1;

    take_while1(char::is_alphanumeric)(cur)
}

fn whitespace(cur: &str) -> Result<&str> {
    use nom::bytes::complete::take_while;

    take_while(char::is_whitespace)(cur)
}

fn command_params(mut cur: &str) -> Result<Params> {
    use nom::{character::complete::char, combinator::opt};

    let mut params = Params::new();

    loop {
        let (i, _) = whitespace(cur)?;

        if let Ok((i, _)) = char::<_, ()>(COMMAND_PARAMS_CHARS.close)(i) {
            cur = i;
            break;
        }

        let (i, ident) = ident(i)?;
        params.insert(ident, None);

        let (i, _) = whitespace
            .and(opt(char(COMMAND_PARAMS_SEP_CHAR)))
            .parse(i)?;

        cur = i;
    }

    Ok((cur, params))
}

fn command(cur: &str) -> Result<CommandType> {
    use nom::{character::complete::char, combinator::recognize};

    if let Ok((i, c)) = recognize(
        char::<_, ()>(COMMENT_CHAR)
            .or(char::<_, ()>(COMMAND_CHAR))
            .or(char::<_, ()>(COMMAND_CONTENT_CHARS.close))
            .or(char::<_, ()>(LINE_BREAK_CHAR)),
    )(cur)
    {
        return Ok((i, CommandType::Escape(c)));
    }

    let (mut cur, mut name) = ident(cur)?;
    let mut namespace = None;
    let mut content = None;
    let mut params = None;

    if let Ok((i, _)) = char::<_, ()>(NAMESPACE_CHAR)(cur) {
        let (i, ident) = ident(i)?;

        cur = i;
        namespace = Some(name);
        name = ident;
    }

    if let Ok((i, _)) = char::<_, ()>(COMMAND_PARAMS_CHARS.open)(cur) {
        let (i, res) = command_params(i)?;

        cur = i;
        params = Some(res);
    }

    if let Ok((i, _)) = char::<_, ()>(COMMAND_CONTENT_CHARS.open)(cur) {
        let (i, (inner, _)) = top_loop.and(char(COMMAND_CONTENT_CHARS.close)).parse(i)?;
        content = Some(inner);
        cur = i;
    }

    let command = Command {
        name,
        namespace,
        content: content.unwrap_or_default(),
        params: params.unwrap_or_default(),
        block: false,
    };

    if namespace == None && name == COMMAND_BLOCK_START {
        return Ok((cur, CommandType::Start(command)));
    }

    if namespace == None && name == COMMAND_BLOCK_END {
        return Ok((cur, CommandType::End(command)));
    }

    Ok((cur, CommandType::Normal(command)))
}

fn raw(cur: &str) -> Result<Element> {
    use nom::bytes::complete::take_till;

    take_till(|c| c == COMMAND_CHAR || c == COMMAND_CONTENT_CHARS.close || c == COMMENT_CHAR)
        .map(Element::Raw)
        .parse(cur)
}

fn comment(cur: &str) -> Result<Element> {
    use nom::bytes::complete::take_till;

    take_till(|c| c == LINE_BREAK_CHAR)
        .map(Element::Comment)
        .parse(cur)
}

fn top(cur: &str) -> Result<Either<Element, CommandType>> {
    use nom::character::complete::char;

    if let Ok((cur, _)) = char::<_, ()>(COMMAND_CHAR)(cur) {
        command.map(Either::Right).parse(cur)
    } else if let Ok((cur, _)) = char::<_, ()>(COMMENT_CHAR)(cur) {
        comment.map(Either::Left).parse(cur)
    } else {
        raw.map(Either::Left).parse(cur)
    }
}

fn block_command(tree: Stream) -> &str {
    // FIXME
    if let Some(Element::Raw(r)) = tree.iter().next() {
        *r
    } else {
        panic!("block_command");
    }
}

fn top_loop(buf: &str) -> Result<Stream> {
    top_loop_ctx(buf, None)
}

fn top_loop_ctx<'b>(mut buf: &'b str, ctx: Option<&'b str>) -> Result<'b, Stream<'b>> {
    use nom::character::complete::char;

    let mut res = Vec::new();

    loop {
        if let Ok(_) = char::<_, ()>(COMMAND_CONTENT_CHARS.close)(buf) {
            // We leave the closing character in the flux to be consumed by the parent, so we
            // can have proper diagnostics in case of mismatched closings.
            break;
        }

        if buf.is_empty() {
            break;
        }

        let (cur, e) = top(buf)?;

        match e {
            Either::Left(e) => res.push(e),
            Either::Right(CommandType::Normal(cmd)) => res.push(Element::Command(cmd)),
            Either::Right(CommandType::Escape(e)) => {
                // TODO: Implement line break
                res.push(Element::Raw(e));
            }
            Either::Right(CommandType::Start(mut cmd)) => {
                let content = std::mem::replace(&mut cmd.content, Vec::new());
                let name = block_command(content);

                let (cur, content) = top_loop_ctx(cur, Some(name))?;

                res.push(Element::Command(Command {
                    name,
                    namespace: None,
                    content,
                    params: cmd.params,
                    block: true,
                }));

                buf = cur;
                continue;
            }
            Either::Right(CommandType::End(mut cmd)) => {
                let content = std::mem::replace(&mut cmd.content, Vec::new());
                let end_name = block_command(content);

                if let Some(start_name) = ctx {
                    if start_name != end_name {
                        panic!(
                            "Closing a {} block while a {} is open",
                            end_name, start_name
                        );
                    }

                    buf = cur;
                    break;
                } else {
                    panic!(
                        "Closing a {} block outside of any block near {:?}",
                        end_name, cur
                    )
                }
            }
        }

        buf = cur;
    }

    Ok((buf, res))
}

pub fn document(buf: &str) -> Result<Stream> {
    use nom::Finish;

    let (buf, res) = top_loop(buf)?;

    if !buf.is_empty() {
        panic!("Extra content at end of file...");
    }

    Ok((buf, res)).finish()
}
