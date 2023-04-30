use super::ast::{EnumDecl, SpawnBlock, StatesBlock, StatesBlockElement, ToplevelElement};
use crate::{Direction, EnemySpawnInfo};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1, take_while},
    character::complete::{alpha1, alphanumeric1, char, multispace0, one_of},
    combinator::recognize,
    error::{ErrorKind, ParseError},
    multi::{many0, many0_count, many1, separated_list0},
    sequence::{delimited, pair, terminated},
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

fn handle_unexpected<'a, Output, P, F>(
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

fn ws<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> Res<O>
where
    F: FnMut(Span<'a>) -> Res<'a, O>,
{
    delimited(multispace0, inner, multispace0)
}

fn is_identifier(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn parse_enum_name(input: Span) -> Res<'_, Span> {
    ws(take_while(is_identifier))(input)
}

fn parse_enum_body(input: Span) -> Res<'_, Vec<Span>> {
    separated_list0(tag(","), parse_enum_name)(input)
}

fn parse_enum_decl(input: Span) -> Res<'_, ToplevelElement> {
    let (input, _) = ws(tag("enum"))(input)?;
    let (input, enum_names) = delimited(char('{'), parse_enum_body, char('}'))(input)?;
    Ok((
        input,
        ToplevelElement::EnumDecl(EnumDecl {
            names: enum_names.iter().map(|v| v.to_string()).collect(),
        }),
    ))
}

fn parse_label(input: Span) -> Res<'_, StatesBlockElement> {
    let (input, label_name) = ws(terminated(take_while(is_identifier), char(':')))(input)?;
    Ok((input, StatesBlockElement::Label(label_name.to_string())))
}

pub fn identifier(input: Span) -> Res<'_, String> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
    .map(|(i, s)| (i, s.to_string()))
}

fn decimal(input: Span) -> Res<'_, i32> {
    recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(input)
        .map(|(i, s)| (i, s.parse::<i32>().expect("failed to parse integer {s}")))
}

fn boolean(input: Span) -> Res<'_, bool> {
    alt((tag("true"), tag("false")))(input).map(|(i, s)| (i, *s.fragment() == "true"))
}

fn parse_state(input: Span) -> Res<'_, StatesBlockElement> {
    let (input, _) = ws(tag("state"))(input)?;
    let (input, (id, directional, ticks, think, action, next)) = parse_state_body(input)?;

    Ok((
        input,
        StatesBlockElement::State {
            id,
            directional,
            ticks,
            think,
            action,
            next,
        },
    ))
}

fn parse_state_body(input: Span) -> Res<'_, (String, bool, i32, String, String, String)> {
    let (input, id) = ws(identifier)(input)?;
    let (input, _) = char(',')(input)?;
    let (input, directional) = ws(boolean)(input)?;
    let (input, _) = char(',')(input)?;
    let (input, ticks) = ws(decimal)(input)?;
    let (input, _) = char(',')(input)?;
    let (input, think) = ws(identifier)(input)?;
    let (input, _) = char(',')(input)?;
    let (input, action) = ws(identifier)(input)?;
    let (input, _) = char(',')(input)?;
    let (input, next) = ws(identifier)(input)?;

    Ok((input, (id, directional, ticks, think, action, next)))
}

fn states_block_element(input: Span) -> Res<'_, StatesBlockElement> {
    alt((parse_label, parse_state))(input)
}

fn parse_states_block(input: Span) -> Res<'_, ToplevelElement> {
    let (input, _) = ws(tag("states"))(input)?;
    let (input, name) = ws(take_while(is_identifier))(input)?;
    let (input, elements) =
        delimited(ws(char('{')), many0(states_block_element), ws(char('}')))(input)?;

    Ok((
        input,
        ToplevelElement::StatesBlock(StatesBlock {
            name: name.to_string(),
            elements,
        }),
    ))
}

fn spawn_on_death(name: &str) -> Option<i32> {
    match name {
        "ammo" => Some(49),
        "silver_key" => Some(43),
        "grofaz" => Some(224), // FIXME: abuse blinky
        _ => None,
    }
}

fn spawn_block_directional_element(input: Span) -> Res<'_, Vec<EnemySpawnInfo>> {
    let (input, _) = ws(tag("directional"))(input)?;
    let (input, start_id) = ws(decimal)(input)?;
    let (input, _) = char(',')(input)?;
    let (input, state) = ws(identifier)(input)?;
    let (input, _) = char(',')(input)?;
    let (input, bonus_item_name) = ws(identifier)(input)?;

    let mut infos = Vec::new();

    for (i, direction) in [
        Direction::East,
        Direction::North,
        Direction::West,
        Direction::South,
    ]
    .iter()
    .enumerate()
    {
        infos.push(EnemySpawnInfo {
            id: start_id + i as i32,
            direction: *direction,
            state: state.clone(),
            spawn_on_death: spawn_on_death(&bonus_item_name),
        })
    }
    Ok((input, infos))
}
fn spawn_block_undirectional_element(input: Span) -> Res<'_, Vec<EnemySpawnInfo>> {
    let (input, _) = ws(tag("undirectional"))(input)?;
    let (input, id) = ws(decimal)(input)?;
    let (input, _) = char(',')(input)?;
    let (input, state) = ws(identifier)(input)?;
    let (input, _) = char(',')(input)?;
    let (input, bonus_item_name) = ws(identifier)(input)?;

    let infos = vec![EnemySpawnInfo {
        id,
        direction: Direction::South, // FIXME: not really undirectional
        state,
        spawn_on_death: spawn_on_death(&bonus_item_name),
    }];
    Ok((input, infos))
}

fn parse_spawn_block(input: Span) -> Res<'_, ToplevelElement> {
    let (input, _) = ws(tag("spawn"))(input)?;
    let (input, name) = ws(take_while(is_identifier))(input)?;
    let (input, mut elements) = delimited(
        ws(char('{')),
        many0(alt((
            spawn_block_directional_element,
            spawn_block_undirectional_element,
        ))),
        ws(char('}')),
    )(input)?;

    let infos = elements.drain(..).flatten().collect();

    Ok((
        input,
        ToplevelElement::SpawnBlock(SpawnBlock {
            name: name.to_string(),
            infos,
        }),
    ))
}

pub fn parse_toplevel(input: Span) -> Res<'_, ToplevelElement> {
    alt((
        parse_enum_decl,
        parse_states_block,
        parse_spawn_block,
        handle_unexpected(take_till1(|c: char| c.is_whitespace()), |txt| {
            MyError::Custom(format!("unexpected at toplevel: {txt:?}"))
        }),
    ))(input)
}
