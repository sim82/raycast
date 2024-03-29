use self::util::{handle_unexpected, is_identifier, ws, MyError, Res, Span};
use super::ast::{
    EnumDecl, FunctionBlock, FunctionBlockElement, SpawnBlock, StatesBlock, StatesBlockElement,
    ToplevelElement,
};
use crate::{Direction, EnemySpawnInfo};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till1, take_while},
    character::complete::{alpha1, alphanumeric1, char, one_of},
    combinator::{recognize, value},
    multi::{many0, many0_count, many1, separated_list0},
    sequence::{delimited, pair, terminated},
};

pub mod util;

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
    recognize(many1(terminated(one_of("-0123456789"), many0(char('_')))))(input)
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

pub fn parse_bytecode_directive(input: Span) -> Res<'_, FunctionBlockElement> {
    // pub fn parse_load_i32(input: Span) -> Res<'_, FunctionBlockElement> {
    //     let (input, _) = ws(tag("loadi32"))(input)?;
    //     let (input, addr) = ws(decimal)(input)?;
    //     Ok((input, FunctionBlockElement::LoadI32 { addr: addr as u8 }))
    // }
    // pub fn parse_store_i32(input: Span) -> Res<'_, FunctionBlockElement> {
    //     let (input, _) = ws(tag("storei32"))(input)?;
    //     let (input, addr) = ws(decimal)(input)?;
    //     Ok((input, FunctionBlockElement::StoreI32 { addr: addr as u8 }))
    // }
    pub fn parse_loadi_i32(input: Span) -> Res<'_, FunctionBlockElement> {
        let (input, _) = ws(tag("loadii32"))(input)?;
        let (input, value) = ws(decimal)(input)?;
        Ok((input, FunctionBlockElement::LoadiI32 { value }))
    }
    pub fn parse_loadi_u8_enum(input: Span) -> Res<'_, FunctionBlockElement> {
        let (input, _) = ws(tag("loadiu8"))(input)?;
        let (input, name) = parse_enum_name(input)?;
        Ok((
            input,
            FunctionBlockElement::LoadiU8Enum {
                name: name.to_string(),
            },
        ))
    }
    pub fn parse_loadi_u8(input: Span) -> Res<'_, FunctionBlockElement> {
        let (input, _) = ws(tag("loadiu8"))(input)?;
        let (input, value) = ws(decimal)(input)?;
        if value > u8::MAX as i32 {
            panic!("u8 out of range {value}"); // FIXME: emit proper parse error
        }
        Ok((input, FunctionBlockElement::LoadiU8 { value: value as u8 }))
    }
    pub fn parse_add(input: Span) -> Res<'_, FunctionBlockElement> {
        let (input, _) = ws(tag("add"))(input)?;
        Ok((input, FunctionBlockElement::Add))
    }
    pub fn parse_trap(input: Span) -> Res<'_, FunctionBlockElement> {
        let (input, _) = ws(tag("trap"))(input)?;
        Ok((input, FunctionBlockElement::Trap))
    }
    pub fn parse_ceq(input: Span) -> Res<'_, FunctionBlockElement> {
        let (input, _) = ws(tag("ceq"))(input)?;
        Ok((input, FunctionBlockElement::Ceq))
    }
    pub fn parse_stop(input: Span) -> Res<'_, FunctionBlockElement> {
        let (input, _) = ws(tag("stop"))(input)?;
        Ok((input, FunctionBlockElement::Stop))
    }
    pub fn parse_not(input: Span) -> Res<'_, FunctionBlockElement> {
        let (input, _) = ws(tag("not"))(input)?;
        Ok((input, FunctionBlockElement::Not))
    }
    pub fn parse_jrc(input: Span) -> Res<'_, FunctionBlockElement> {
        let (input, _) = ws(tag("jrc"))(input)?;
        let (input, label) = ws(take_while(is_identifier))(input)?;
        Ok((
            input,
            FunctionBlockElement::Jrc {
                label: label.to_string(),
            },
        ))
    }
    pub fn parse_load_state_label(input: Span) -> Res<'_, FunctionBlockElement> {
        let (input, _) = ws(tag("loadsl"))(input)?;
        let (input, label) = ws(take_while(is_identifier))(input)?;
        Ok((
            input,
            FunctionBlockElement::LoadStateLabel {
                label: label.to_string(),
            },
        ))
    }
    pub fn parse_gostate(input: Span) -> Res<'_, FunctionBlockElement> {
        let (input, _) = ws(tag("gostate"))(input)?;
        Ok((input, FunctionBlockElement::GoState))
    }
    pub fn parse_label(input: Span) -> Res<'_, FunctionBlockElement> {
        let (input, label) = ws(terminated(take_while(is_identifier), char(':')))(input)?;
        Ok((input, FunctionBlockElement::Label(label.to_string())))
    }
    pub fn parse_call(input: Span) -> Res<'_, FunctionBlockElement> {
        let (input, _) = ws(tag("call"))(input)?;
        Ok((input, FunctionBlockElement::FunctionCall))
    }
    pub fn parse_line_comment(input: Span) -> Res<'_, ()> {
        value(
            (), // Output is thrown away.
            pair(tag("//"), is_not("\n\r")),
        )(input)
    }
    let (input, _) = many0(parse_line_comment)(input)?;
    let (input, e) = ws(alt((
        // parse_load_i32,
        // parse_store_i32,
        parse_trap,
        parse_loadi_i32,
        parse_loadi_u8, // FIXME: identifier parsing is crap (accepts also digit in fist place). need to match this rule first.
        parse_loadi_u8_enum,
        parse_jrc,
        parse_load_state_label,
        parse_gostate,
        parse_add,
        parse_ceq,
        parse_not,
        parse_call,
        parse_label,
        parse_stop,
    )))(input)?;
    let (input, _) = many0(parse_line_comment)(input)?;
    Ok((input, e))
}
pub fn parse_function_block(input: Span) -> Res<'_, ToplevelElement> {
    let (input, _) = ws(tag("function"))(input)?;
    let (input, name) = ws(take_while(is_identifier))(input)?;
    let (input, elements) = delimited(
        ws(char('{')),
        many0(alt((parse_bytecode_directive,))),
        ws(char('}')),
    )(input)?;

    Ok((
        input,
        ToplevelElement::FunctionBlock(FunctionBlock {
            name: name.to_string(),
            elements,
        }),
    ))
}

pub fn parse_toplevel(input: Span) -> Res<'_, ToplevelElement> {
    alt((
        parse_enum_decl,
        parse_states_block,
        parse_spawn_block,
        parse_function_block,
        handle_unexpected(take_till1(|c: char| c.is_whitespace()), |txt| {
            MyError::Custom(format!("unexpected at toplevel: {txt:?}"))
        }),
    ))(input)
}
