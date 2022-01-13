use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{
        alpha1, alphanumeric1, digit1, line_ending, not_line_ending, space0, space1,
    },
    combinator::{map, map_res, opt, recognize},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

#[derive(Debug, PartialEq)]
pub enum Command<'s> {
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
    Return,
    Label(&'s str),
    Goto(&'s str),
    IfGoto(&'s str),
    Call(&'s str, u16),
    Function(&'s str, u16),
    Pop(Segment, u16),
    Push(Segment, u16),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Segment {
    Local,
    Argument,
    This,
    That,
    Constant,
    Static,
    Pointer,
    Temp,
}

type Result<'s, T> = IResult<&'s str, T, nom::error::Error<&'s str>>;

fn comment(input: &str) -> Result<()> {
    map(preceded(tag("//"), not_line_ending), |_| ())(input)
}

fn number(input: &str) -> IResult<&str, u16> {
    map_res(digit1, |s: &str| s.parse())(input)
}

// Based on https://github.com/Geal/nom/blob/e99f9e0/doc/nom_recipes.md#identifiers
fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_"), tag(".")))),
    ))(input)
}

fn segment(input: &str) -> IResult<&str, Segment> {
    alt((
        map(tag("local"), |_| Segment::Local),
        map(tag("argument"), |_| Segment::Argument),
        map(tag("this"), |_| Segment::This),
        map(tag("that"), |_| Segment::That),
        map(tag("constant"), |_| Segment::Constant),
        map(tag("static"), |_| Segment::Static),
        map(tag("pointer"), |_| Segment::Pointer),
        map(tag("temp"), |_| Segment::Temp),
    ))(input)
}

// https://en.wikipedia.org/wiki/Arity#Nullary
fn nullary_cmd(input: &str) -> IResult<&str, Command> {
    alt((
        map(tag("add"), |_| Command::Add),
        map(tag("sub"), |_| Command::Sub),
        map(tag("neg"), |_| Command::Neg),
        map(tag("eq"), |_| Command::Eq),
        map(tag("gt"), |_| Command::Gt),
        map(tag("lt"), |_| Command::Lt),
        map(tag("and"), |_| Command::And),
        map(tag("or"), |_| Command::Or),
        map(tag("not"), |_| Command::Not),
        map(tag("return"), |_| Command::Return),
    ))(input)
}

fn unary_cmd(input: &str) -> IResult<&str, Command> {
    map(
        pair(
            alt((tag("label"), tag("goto"), tag("if-goto"))),
            preceded(space1, identifier),
        ),
        |(cmd, ident)| match cmd {
            "label" => Command::Label(ident),
            "goto" => Command::Goto(ident),
            "if-goto" => Command::IfGoto(ident),
            _ => unreachable!("no other strings are possible"),
        },
    )(input)
}

fn binary_cmd(input: &str) -> IResult<&str, Command> {
    alt((pop_or_push, call_or_function))(input)
}

fn pop_or_push(input: &str) -> IResult<&str, Command> {
    map(
        tuple((
            alt((tag("pop"), tag("push"))),
            delimited(space1, segment, space1),
            number,
        )),
        |(cmd, seg, n)| match cmd {
            "pop" => Command::Pop(seg, n),
            "push" => Command::Push(seg, n),
            _ => unreachable!("no other strings are possible"),
        },
    )(input)
}

fn call_or_function(input: &str) -> IResult<&str, Command> {
    map(
        tuple((
            alt((tag("call"), tag("function"))),
            delimited(space1, identifier, space1),
            number,
        )),
        |(cmd, ident, n)| match cmd {
            "call" => Command::Call(ident, n),
            "function" => Command::Function(ident, n),
            _ => unreachable!("no other strings are possible"),
        },
    )(input)
}

fn command(input: &str) -> IResult<&str, Command> {
    alt((nullary_cmd, unary_cmd, binary_cmd))(input)
}

fn line(input: &str) -> IResult<&str, Option<Command>> {
    preceded(
        space0,
        alt((
            map(terminated(command, pair(space0, opt(comment))), Some),
            map(opt(comment), |_| None),
        )),
    )(input)
}

pub fn parse(input: &str) -> Result<Vec<Option<Command>>> {
    separated_list0(line_ending, line)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comments() {
        assert_eq!(comment("// hi"), Ok(("", ())));
        assert!(comment("no-comment").is_err());
    }

    #[test]
    fn test_number() {
        assert_eq!(number("0"), Ok(("", 0)));
        assert_eq!(number("1337"), Ok(("", 1337)));
        assert!(number("-1").is_err());
        assert!(number("65536").is_err()); // u16::MAX + 1
    }

    #[test]
    fn test_segment() {
        assert_eq!(segment("local"), Ok(("", Segment::Local)));
        assert_eq!(segment("that"), Ok(("", Segment::That)));
        assert!(segment("foo").is_err());
    }

    #[test]
    fn test_identifier() {
        assert_eq!(identifier("FOO_0"), Ok(("", "FOO_0")));
        assert_eq!(identifier("bar1"), Ok(("", "bar1")));
        assert_eq!(identifier("Foo.bar"), Ok(("", "Foo.bar")));
        assert!(identifier("1foo").is_err());
    }

    #[test]
    fn test_nullary_cmd() {
        assert_eq!(nullary_cmd("add"), Ok(("", Command::Add)));
        assert_eq!(nullary_cmd("lt"), Ok(("", Command::Lt)));
        assert_eq!(nullary_cmd("return"), Ok(("", Command::Return)));
        assert!(nullary_cmd("push").is_err());
    }

    #[test]
    fn test_unary_cmd() {
        assert_eq!(unary_cmd("label   FOO"), Ok(("", Command::Label("FOO"))));
        assert_eq!(unary_cmd("goto    FOO"), Ok(("", Command::Goto("FOO"))));
        assert_eq!(unary_cmd("if-goto FOO"), Ok(("", Command::IfGoto("FOO"))));
        assert!(unary_cmd("label 1").is_err());
        assert!(unary_cmd("call").is_err());
    }

    #[test]
    fn test_binary_cmd() {
        assert_eq!(
            binary_cmd("push this 0"),
            Ok(("", Command::Push(Segment::This, 0)))
        );
        assert_eq!(
            binary_cmd("pop    that    42"),
            Ok(("", Command::Pop(Segment::That, 42)))
        );
        assert_eq!(binary_cmd("call A.b 1"), Ok(("", Command::Call("A.b", 1))));
        assert_eq!(
            binary_cmd("function C.d 0"),
            Ok(("", Command::Function("C.d", 0)))
        );
        assert!(binary_cmd("add").is_err());
        assert!(binary_cmd("pop").is_err());
        assert!(binary_cmd("push that").is_err());
        assert!(binary_cmd("nope that 1").is_err());
        assert!(binary_cmd("pop this -20").is_err());
        assert!(binary_cmd("pop invalidsegment 2").is_err());
        assert!(binary_cmd("function 1error 2").is_err());
    }

    #[test]
    fn test_command() {
        assert_eq!(command("sub"), Ok(("", Command::Sub)));
        assert_eq!(command("label FOO"), Ok(("", Command::Label("FOO"))));
        assert_eq!(
            command("push this 0"),
            Ok(("", Command::Push(Segment::This, 0)))
        );
        assert!(command("").is_err());
        assert!(command("// add").is_err());
    }

    #[test]
    fn test_line() {
        use Command::*;
        assert_eq!(line("add"), Ok(("", Some(Add))));
        assert_eq!(line("add\n"), Ok(("\n", Some(Add))));
        assert_eq!(line("  add  \n"), Ok(("\n", Some(Add))));
        assert_eq!(line("add// hi\n"), Ok(("\n", Some(Add))));
        assert_eq!(line("add //hi\n"), Ok(("\n", Some(Add))));
        assert_eq!(line("\n"), Ok(("\n", None)));
        assert_eq!(line("  \n"), Ok(("\n", None)));
        assert_eq!(line("// hi\n"), Ok(("\n", None)));
        assert_eq!(line("    // hi\n"), Ok(("\n", None)));
        assert_eq!(line("foo\n"), Ok(("foo\n", None)));
    }

    #[test]
    fn test_parse() {
        use Command::*;
        assert_eq!(parse(""), Ok(("", vec![None])));
        assert_eq!(parse("\n"), Ok(("", vec![None, None])));
        assert_eq!(parse("\nadd"), Ok(("", vec![None, Some(Add)])));
        assert_eq!(parse("add"), Ok(("", vec![Some(Add)])));
        assert_eq!(parse("add\nfoo"), Ok(("foo", vec![Some(Add), None])));
        assert_eq!(parse("add\n  sub"), Ok(("", vec![Some(Add), Some(Sub)])));
        assert_eq!(parse("add\n// hi\n"), Ok(("", vec![Some(Add), None, None])));
        assert_eq!(parse("pop this \n0"), Ok(("pop this \n0", vec![None])));
    }
}
