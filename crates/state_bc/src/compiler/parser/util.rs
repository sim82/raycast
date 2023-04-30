use nom::{
    character::complete::multispace0,
    error::{ErrorKind, ParseError},
    sequence::delimited,
    IResult,
};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;
pub type Res<'a, Output> = IResult<Span<'a>, Output, MyError<Span<'a>>>;

// WIP: error handling inspired by https://www.reddit.com/r/rust/comments/mtmufz/comment/gv18j0a/?utm_source=share&utm_medium=web2x&context=3
// but much more primitive and simplified by using nom_locate.

#[derive(Debug)]
pub enum MyError<I> {
    Nom(nom::error::Error<I>),
    Custom(String),
}

impl<I> ParseError<I> for MyError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        // Error { input, code: kind }
        MyError::Nom(nom::error::Error::from_error_kind(input, kind))
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

pub fn handle_unexpected<'a, Output, P, F>(
    parser: P,
    error_f: F,
) -> impl Fn(Span<'a>) -> Res<'a, Output>
where
    P: Fn(Span<'a>) -> Res<'a, Span<'a>>,
    F: Fn(Span<'a>) -> MyError<Span<'a>>,
{
    use nom::Err::{Error, Failure, Incomplete};
    move |i: Span<'a>| match parser(i) {
        Ok((_i_after_parse, parsed)) => Err(Failure(error_f(parsed))),
        Err(Failure(e)) => Err(Failure(e)),
        Err(Error(e)) => Err(Error(e)),
        Err(Incomplete(need)) => Err(Incomplete(need)),
    }
}

pub fn ws<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> Res<O>
where
    F: FnMut(Span<'a>) -> Res<'a, O>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn is_identifier(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
