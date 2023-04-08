use std::{
    collections::{BTreeMap, HashMap},
    io::Write,
};

use byteorder::{LittleEndian, WriteBytesExt};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{alpha1, alphanumeric1, char, multispace0, one_of},
    combinator::recognize,
    error::ParseError,
    multi::{many0, many0_count, many1, separated_list0},
    sequence::{delimited, pair, terminated},
    IResult,
};
use raycast::{ms::Writable, prelude::*, state_bc};

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

fn codegen(outname: &str, state_blocks: &[StatesBlock], enums: &BTreeMap<String, usize>) {
    let _ = std::fs::rename(outname, format!("{outname}.bak")); // don't care if it does not work

    let mut states = Vec::new();
    let mut label_ptrs = BTreeMap::new(); // keep them sorted in the output file

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
                } => ip += state_bc::STATE_BC_SIZE,
            }
        }
    }

    // pass2: generate code
    // calculate size of labels section
    let mut ip = label_ptrs.keys().map(|k| k.len() as i32 + 1 + 4).sum::<i32>() + 4;
    // 're-locate' label pointers to point after the labels section
    println!("bc offset: {ip}");
    label_ptrs.values_mut().for_each(|i| *i += ip);

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
                    ip + state_bc::STATE_BC_SIZE
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
                ip += state_bc::STATE_BC_SIZE;
            }
        }
    }

    let mut f = std::fs::File::create(outname).expect("failed to open img file");
    f.write_i32::<LittleEndian>(label_ptrs.len() as i32).unwrap();
    for (name, ptr) in &label_ptrs {
        let b = name.as_bytes();
        f.write_u8(b.len() as u8).unwrap();
        let _ = f.write(b).unwrap();
        f.write_i32::<LittleEndian>(*ptr).unwrap();
    }
    for state in states {
        state.write(&mut f).unwrap();
    }
}

fn ws<'a, F, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: FnMut(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn is_identifier(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn parse_enum_name(input: &str) -> IResult<&str, &str> {
    ws(take_while(is_identifier))(input)
}

fn parse_enum_body(input: &str) -> IResult<&str, Vec<&str>> {
    separated_list0(tag(","), parse_enum_name)(input)
}

fn parse_enum_decl(input: &str) -> IResult<&str, ToplevelElement> {
    let (input, _) = ws(tag("enum"))(input)?;
    let (input, enum_names) = delimited(char('{'), parse_enum_body, char('}'))(input)?;
    Ok((
        input,
        ToplevelElement::EnumDecl(EnumDecl {
            names: enum_names.iter().map(|v| v.to_string()).collect(),
        }),
    ))
}

fn parse_label(input: &str) -> IResult<&str, StatesBlockElement> {
    let (input, label_name) = ws(terminated(take_while(is_identifier), char(':')))(input)?;
    Ok((input, StatesBlockElement::Label(label_name.to_string())))
}

pub fn identifier(input: &str) -> IResult<&str, String> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
    .map(|(i, s)| (i, s.to_string()))
}

fn decimal(input: &str) -> IResult<&str, i32> {
    recognize(many1(terminated(one_of("0123456789"), many0(char('_')))))(input)
        .map(|(i, s)| (i, s.parse::<i32>().expect("failed to parse integer {s}")))
}

fn boolean(input: &str) -> IResult<&str, bool> {
    alt((tag("true"), tag("false")))(input).map(|(i, s)| (i, s == "true"))
}

fn parse_state(input: &str) -> IResult<&str, StatesBlockElement> {
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

fn parse_state_body(input: &str) -> IResult<&str, (String, bool, i32, String, String, String)> {
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

fn states_block_element(input: &str) -> IResult<&str, StatesBlockElement> {
    alt((parse_label, parse_state))(input)
}

fn parse_states_block(input: &str) -> IResult<&str, ToplevelElement> {
    let (input, _) = ws(tag("states"))(input)?;
    let (input, name) = ws(take_while(is_identifier))(input)?;
    let (input, elements) = delimited(ws(char('{')), many0(states_block_element), ws(char('}')))(input)?;

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
    let outname = std::env::args().nth(2).expect("missing output file");

    let input = std::fs::read_to_string(filename).expect("failed to read input file");

    let (_input, toplevel_elements) = many1(parse_toplevel)(&input).expect("failed to parse toplevel elements");
    // println!("elements: {toplevel_elements:?}");

    let mut enums = BTreeMap::new();
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

    {
        let mut enum_file = std::fs::File::create(format!("{outname}.enums")).unwrap();
        // write!(enum_file, "{enums:?}").unwrap();
        let _ = writeln!(enum_file, "const ENUM_NAMES: [(&str, i32); {}] = [", enums.len());
        // let _ = writeln!(enum_file, "[");
        for (name, id) in enums.iter() {
            let _ = write!(enum_file, "(\"{name}\", {id}), ");
        }
        let _ = write!(enum_file, "\n];");
    }
    codegen(&outname, &state_blocks, &enums);
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
