//! The parser used to transform pastex documents into a syntax tree for processing by an engine.

use either::Either;
use nom::Parser;
use std::collections::HashMap;
use std::fmt;

/// A command parameters can take different forms. Depending on the form read from the file, it will
/// have a different associated value from this enum.
///
/// * For simple arguments like `[foo]`, you will get [`ParamValue::None`] associated.
/// * For aguments with a simple textual value, like `[foo = bar]`, you will obtain a
///   [`ParamValue::Text`] with the text span accessible directly.
/// * For arguments used with an evaluation span, like `[foo = { bar }]`, you will obtain a
///   [`ParamValue::Stream`] and a [`Stream`] value to work with. You will have to process it
///   like you would a top-level stream.
#[derive(Debug)]
pub enum ParamValue<'b> {
    /// Simple parameter without value
    None,
    /// Parameter with a direct textual value
    Text(&'b str),
    /// Parameter with an evaluation span container an inner stream to process
    Stream(Stream<'b>),
}

/// Represents parameters passed to a command. See [`ParamValue`] for a more detailled description
/// of possible values.
pub type Params<'b> = HashMap<&'b str, ParamValue<'b>>;

/// A stream is a list of recognized elements of the same level.
pub type Stream<'b> = Vec<Element<'b>>;

/// Represents a command call itself, used inside an element to hold all associated data and be
/// able to pass the command details themselves around.
///
/// Command calls look like this:
///
/// ```tex
/// % Calls a function named `foo`:
/// \foo
/// \foo{}
/// % Calls a function `foo` in the namespace `bar`:
/// \bar:foo
/// % Calls a function `foo` with some content:
/// \foo{contents here...}
/// % Can provide parameters, too:
/// \foo[bar]{...}
/// \foo[bar, baz = 1]{...}
/// \foo[bar = {some more content and \commands}]{...}
/// ```
///
/// To use a function with a large block of text, you can use the `begin` and `end` special commands
/// to delimitate the call:
///
/// ```tex
/// \begin{foo} % Command name goes in \begin's "content"
/// ...
/// \end{foo}
///
/// \begin[bar, baz = 1]{foo} % Parameters to `begin` are passed to `foo`
/// ...
/// \end{foo}
/// ```
///
/// `begin` and `end` commands are converted into a command call to `foo`, like if you used
/// `\foo{ ... }`. However, such uses are marked, and your engine can choose to act differently
/// on block commands with the same name.
///
/// All forms given above will all be saved into the given structure below, filling different fields
/// with the appropriate information.
#[derive(Debug)]
pub struct Command<'b> {
    /// The name of the command
    pub name: &'b str,
    /// The namespace of a command, if one is indicated, otherwise [`None`]
    pub namespace: Option<&'b str>,
    /// The contents inside of the command call. Empty list if the call is done without any
    /// contents
    pub content: Stream<'b>,
    /// Parameters given to the command. Check [`Params`] and [`ParamValue`]
    pub params: Params<'b>,
    /// `true` when the block (`begin`/`end`) form has been used, `false` for standard syntax
    pub block: bool,
}

/// Helper value to represent the name of a function call. Only holds the name and optionally
/// namespace of a function call.
///
/// This structure is obtained with [`Command::command_name`] to obtain a displayable name you
/// can use in `format!` and other cases where you need a string. You can also use it to compare
/// if two command calls call into the same function.
///
/// If you need to get the inner name and/or namespace, use the source [`Command`] value.
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
    /// Gets a displayable object from the command name.
    pub fn command_name(&self) -> CommandName<'b> {
        CommandName(self.name, self.namespace)
    }
}

/// Any recognized pastex syntax element from a stream.
#[derive(Debug)]
pub enum Element<'b> {
    /// A command call. See [`Command`] for more details.
    Command(Command<'b>),
    /// Raw, unprocessed text
    Raw(&'b str),
    /// A comment, usually ignored
    Comment(&'b str),
    /// A forced line break, obtained by putting a backslash before a line break.
    LineBreak,
}

enum CommandType<'b> {
    Normal(Command<'b>),
    Start(Command<'b>),
    End(Command<'b>),
    Escape(&'b str),
}

type Result<'t, T> = nom::IResult<&'t str, T>;

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
        params.insert(ident, ParamValue::None);

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

/// Parses a pastex document
///
/// Reads the whole document from a text buffer `buf`, then returns, as a [`Stream`], a tree
/// structure of the document and all function calls inside for processing by a compatible
/// engine.
pub fn document(buf: &str) -> std::result::Result<Stream, nom::error::Error<&str>> {
    use nom::Finish;

    match top_loop(buf).finish() {
        Ok((buf, _)) if !buf.is_empty() => panic!("Extra content at end of file..."),
        Ok((_, res)) => Ok(res),
        Err(e) => Err(e),
    }
}
