use std::{collections::HashMap, io::Write};

use byteorder::{LittleEndian, WriteBytesExt};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{char, digit1, multispace0, multispace1},
    multi::{many0, many1, separated_list0},
    sequence::{delimited, terminated},
    IResult,
};
use raycast::{enemy, ms::Writable, prelude::*};

#[derive(Debug)]
enum ToplevelElement {
    EnumDecl(EnumDecl),
    StatesBlock(StatesBlock),
}

#[derive(Debug)]
struct EnumDecl {
    names: Vec<String>,
}

#[derive(Debug)]
enum StatesBlockElement {
    Label(String),
    State {
        id: String,
        directional: bool,
        ticks: i32,
        think: String,
        action: String,
        next: String,
    },
}

#[derive(Debug)]
struct StatesBlock {
    // bc_name: String,
    // lb_name: String,
    name: String,
    elements: Vec<StatesBlockElement>,
}
// impl StatesBlock {
//     fn codegen(&self, enums: &HashMap<String, usize>) {
//         let mut states = Vec::new();
//         let mut label_ptrs = HashMap::new();
//         // pass 1: resolve label offsets
//         let mut ip = 0;
//         for element in &self.elements {
//             match element {
//                 StatesBlockElement::Label(name) => {
//                     label_ptrs.insert(name, ip);
//                 }
//                 StatesBlockElement::State {
//                     id: _,
//                     directional: _,
//                     ticks: _,
//                     think: _,
//                     action: _,
//                     next: _,
//                 } => ip += enemy::STATE_BC_SIZE,
//             }
//         }
//         // pass2: generate code
//         let mut ip = 0;
//         for element in &self.elements {
//             if let StatesBlockElement::State {
//                 id,
//                 directional,
//                 ticks,
//                 think,
//                 action,
//                 next,
//             } = element
//             {
//                 let id = *enums.get(id).unwrap_or_else(|| panic!("unknown identifier {id}")) as i32;
//                 let next_ptr = if next == "next" {
//                     ip + enemy::STATE_BC_SIZE
//                 } else {
//                     *label_ptrs
//                         .get(next)
//                         .unwrap_or_else(|| panic!("unknown label name {next}"))
//                 };
//                 states.push(StateBc {
//                     id,
//                     ticks: *ticks,
//                     directional: *directional,
//                     think: Think::from_identifier(think),
//                     action: Action::from_identifier(action),
//                     next: next_ptr,
//                 });
//                 ip += enemy::STATE_BC_SIZE;
//             }
//         }

//         let mut f = std::fs::File::create(&self.lb_name).expect("failed to open lb file");
//         f.write_i32::<LittleEndian>(label_ptrs.len() as i32).unwrap();
//         for (name, ptr) in &label_ptrs {
//             let b = name.as_bytes();
//             f.write_u8(b.len() as u8).unwrap();
//             let _ = f.write(b).unwrap();
//             f.write_i32::<LittleEndian>(*ptr).unwrap();
//         }
//         let mut f = std::fs::File::create(&self.bc_name).expect("failed to open bc file");
//         for state in states {
//             state.write(&mut f).unwrap();
//         }
//     }
// }

fn codegen(basename: &str, state_blocks: &[StatesBlock], enums: &HashMap<String, usize>) {
    let mut states = Vec::new();
    let mut label_ptrs = HashMap::new();
    // pass 1: resolve label offsets
    let mut ip = 0;
    for state_block in state_blocks {
        for element in &state_block.elements {
            match element {
                StatesBlockElement::Label(name) => {
                    label_ptrs.insert(format!("{}::{}", state_block.name, name), ip);
                }
                StatesBlockElement::State {
                    id: _,
                    directional: _,
                    ticks: _,
                    think: _,
                    action: _,
                    next: _,
                } => ip += enemy::STATE_BC_SIZE,
            }
        }
    }
    // pass2: generate code
    let mut ip = 0;
    for state_block in state_blocks {
        for element in &state_block.elements {
            if let StatesBlockElement::State {
                id,
                directional,
                ticks,
                think,
                action,
                next,
            } = element
            {
                let id = *enums.get(id).unwrap_or_else(|| panic!("unknown identifier {id}")) as i32;
                let next_ptr = if next == "next" {
                    ip + enemy::STATE_BC_SIZE
                } else {
                    let label = format!("{}::{}", state_block.name, next);
                    *label_ptrs
                        .get(&label)
                        .unwrap_or_else(|| panic!("unknown label name {label}"))
                };
                states.push(StateBc {
                    id,
                    ticks: *ticks,
                    directional: *directional,
                    think: Think::from_identifier(think),
                    action: Action::from_identifier(action),
                    next: next_ptr,
                });
                ip += enemy::STATE_BC_SIZE;
            }
        }
    }

    let mut f = std::fs::File::create("out.lb").expect("failed to open lb file");
    f.write_i32::<LittleEndian>(label_ptrs.len() as i32).unwrap();
    for (name, ptr) in &label_ptrs {
        let b = name.as_bytes();
        f.write_u8(b.len() as u8).unwrap();
        let _ = f.write(b).unwrap();
        f.write_i32::<LittleEndian>(*ptr).unwrap();
    }
    let mut f = std::fs::File::create("out.bc").expect("failed to open bc file");
    for state in states {
        state.write(&mut f).unwrap();
    }
}

fn is_identifier(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_filename(c: char) -> bool {
    is_identifier(c) || c == '.'
}

fn parse_enum_name(input: &str) -> IResult<&str, &str> {
    delimited(multispace0, take_while(is_identifier), multispace0)(input)
}

fn parse_enum_body(input: &str) -> IResult<&str, Vec<&str>> {
    separated_list0(tag(","), parse_enum_name)(input)
}

fn parse_enum_decl(input: &str) -> IResult<&str, ToplevelElement> {
    let (input, _) = delimited(multispace0, tag("enum"), multispace0)(input)?;
    let (input, enum_names) = delimited(char('{'), parse_enum_body, char('}'))(input)?;
    Ok((
        input,
        ToplevelElement::EnumDecl(EnumDecl {
            names: enum_names.iter().map(|v| v.to_string()).collect(),
        }),
    ))
}

fn parse_label(input: &str) -> IResult<&str, StatesBlockElement> {
    let (input, label_name) = delimited(
        multispace0,
        terminated(take_while(is_identifier), char(':')),
        multispace0,
    )(input)?;
    Ok((input, StatesBlockElement::Label(label_name.to_string())))
}

fn parse_wd_identifier(input: &str) -> IResult<&str, String> {
    delimited(multispace0, take_while(is_identifier), multispace0)(input).map(|(i, v)| (i, v.to_string()))
}

fn parse_wd_integer(input: &str) -> IResult<&str, i32> {
    let (input, v) = delimited(multispace0, digit1, multispace0)(input)?;
    Ok((input, v.parse::<i32>().unwrap())) // parse cannot fail
}

fn parse_wd_bool(input: &str) -> IResult<&str, bool> {
    let (input, v) = delimited(multispace0, alt((tag("true"), tag("false"))), multispace0)(input)?;
    Ok((input, v == "true"))
}

fn parse_state(input: &str) -> IResult<&str, StatesBlockElement> {
    let (input, _) = delimited(multispace0, tag("state"), multispace1)(input)?;
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

fn parse_state_body(input: &str) -> IResult<&str, (String, bool, i32, String, String, String)> {
    let (input, id) = parse_wd_identifier(input)?;
    let (input, _) = char(',')(input)?;
    let (input, directional) = parse_wd_bool(input)?;
    let (input, _) = char(',')(input)?;
    let (input, ticks) = parse_wd_integer(input)?;
    let (input, _) = char(',')(input)?;
    let (input, think) = parse_wd_identifier(input)?;
    let (input, _) = char(',')(input)?;
    let (input, action) = parse_wd_identifier(input)?;
    let (input, _) = char(',')(input)?;
    let (input, next) = parse_wd_identifier(input)?;

    Ok((input, (id, directional, ticks, think, action, next)))
}

fn states_block_element(input: &str) -> IResult<&str, StatesBlockElement> {
    alt((parse_label, parse_state))(input)
}

fn parse_states_block(input: &str) -> IResult<&str, ToplevelElement> {
    let (input, _) = delimited(multispace0, tag("states"), multispace0)(input)?;
    let (input, name) = delimited(multispace0, take_while(is_identifier), multispace0)(input)?;
    let (input, elements) = delimited(
        delimited(multispace0, char('{'), multispace0),
        many0(states_block_element),
        delimited(multispace0, char('}'), multispace0),
    )(input)?;

    Ok((
        input,
        ToplevelElement::StatesBlock(StatesBlock {
            name: name.to_string(),
            elements,
        }),
    ))
}

fn parse_toplevel(input: &str) -> IResult<&str, ToplevelElement> {
    alt((parse_enum_decl, parse_states_block))(input)
}

fn main() {
    let filename = std::env::args().nth(1).expect("missing input file");

    let input = std::fs::read_to_string(filename).expect("failed to read input file");

    let (_input, toplevel_elements) = many1(parse_toplevel)(&input).expect("failed to parse toplevel elements");
    println!("elements: {toplevel_elements:?}");

    let mut enums = HashMap::new();
    let mut state_blocks = Vec::new();
    for tle in toplevel_elements {
        match tle {
            ToplevelElement::EnumDecl(mut enum_decl) => {
                for (i, name) in enum_decl.names.drain(..).enumerate() {
                    enums.insert(name, i + 1);
                }
            }
            ToplevelElement::StatesBlock(state_block) => state_blocks.push(state_block),
        }
    }
    //     for states_block in state_blocks {
    //         states_block.codegen(&enums);
    //     }

    codegen("out", &state_blocks, &enums);
}

#[test]
fn test_enum_parser() {
    let (input, decl) = parse_enum_decl("enum {test,\ntest2}").unwrap();
}

#[test]
fn test_states_block() {
    let input = r#"
    states brown_gen.bc brown_gen.lb {
        stand:
            state SPR_GRD_S_1, true, 0, Stand, None, next
        path:
            state SPR_GRD_W1_1, true, 15, Path, None, next
            state SPR_GRD_W2_1, true, 15, Path, None, next
            state SPR_GRD_W3_1, true, 15, Path, None, path
        }
    "#;

    let (input, decl) = parse_states_block(input).unwrap();
}
