use either::Either;
use nom::Parser;
use std::collections::HashMap;
use std::fmt;

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

#[derive(PartialEq)]
pub struct CommandName<'b>(&'b str, Option<&'b str>);

impl<'b> fmt::Display for CommandName<'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.1
            .and_then(|namespace| Some(write!(f, "{}:", namespace)))
            .transpose()
            .and_then(|_| self.0.fmt(f))
    }
}

impl<'b> Command<'b> {
    pub fn command_name(&self) -> CommandName<'b> {
        CommandName(self.name, self.namespace)
    }
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

fn command_name(cur: &str) -> Result<CommandName> {
    use nom::{character::complete::char, combinator::opt};

    ident
        .and(opt(char(NAMESPACE_CHAR).and(ident).map(|(_, i)| i)))
        .map(|(left, right)| {
            if let Some(right) = right {
                CommandName(right, Some(left))
            } else {
                CommandName(left, None)
            }
        })
        .parse(cur)
}

fn command(cur: &str) -> Result<CommandType> {
    use nom::{character::complete::char, combinator::recognize, sequence::tuple};

    if let Ok((i, c)) = recognize(
        char::<_, ()>(COMMENT_CHAR)
            .or(char::<_, ()>(COMMAND_CHAR))
            .or(char::<_, ()>(COMMAND_CONTENT_CHARS.close))
            .or(char::<_, ()>(LINE_BREAK_CHAR)),
    )(cur)
    {
        return Ok((i, CommandType::Escape(c)));
    }

    let (mut cur, name) = command_name(cur)?;
    let mut content = None;
    let mut params = None;

    if let Ok((i, _)) = char::<_, ()>(COMMAND_PARAMS_CHARS.open)(cur) {
        let (i, res) = command_params(i)?;

        cur = i;
        params = Some(res);
    }

    if name == CommandName(COMMAND_BLOCK_START, None)
        || name == CommandName(COMMAND_BLOCK_END, None)
    {
        let (i, (_, real_name, _)) = tuple((
            char(COMMAND_CONTENT_CHARS.open),
            command_name,
            char(COMMAND_CONTENT_CHARS.close),
        ))(cur)?;

        let command = Command {
            name: real_name.0,
            namespace: real_name.1,
            params: params.unwrap_or_default(),
            content: Vec::new(),
            block: false,
        };

        if name.0 == COMMAND_BLOCK_START {
            return Ok((i, CommandType::Start(command)));
        } else {
            return Ok((i, CommandType::End(command)));
        }
    } else if let Ok((i, _)) = char::<_, ()>(COMMAND_CONTENT_CHARS.open)(cur) {
        let (i, (inner, _)) = top_loop.and(char(COMMAND_CONTENT_CHARS.close)).parse(i)?;
        content = Some(inner);
        cur = i;
    }

    let command = Command {
        name: name.0,
        namespace: name.1,
        content: content.unwrap_or_default(),
        params: params.unwrap_or_default(),
        block: false,
    };
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

fn top_loop(buf: &str) -> Result<Stream> {
    top_loop_ctx(buf, None)
}

fn top_loop_ctx<'b>(mut buf: &'b str, ctx: Option<CommandName>) -> Result<'b, Stream<'b>> {
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
            Either::Right(CommandType::Start(cmd)) => {
                let (cur, content) = top_loop_ctx(cur, Some(cmd.command_name()))?;

                res.push(Element::Command(Command {
                    name: cmd.name,
                    namespace: cmd.namespace,
                    content,
                    params: cmd.params,
                    block: true,
                }));

                buf = cur;
                continue;
            }
            Either::Right(CommandType::End(cmd)) => {
                if let Some(start_name) = ctx {
                    if start_name != cmd.command_name() {
                        panic!(
                            "Closing a {} block while a {} is open",
                            cmd.command_name(),
                            start_name
                        );
                    }

                    buf = cur;
                    break;
                } else {
                    panic!(
                        "Closing a {} block outside of any block near {:?}",
                        cmd.command_name(),
                        cur
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
