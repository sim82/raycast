use crate::{
    ms::Writable, opcode::Codegen, Direction, EnemySpawnInfo, Function, SpawnInfos, StateBc,
};
use byteorder::{LittleEndian, WriteBytesExt};
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
use std::{
    collections::{BTreeMap, HashMap},
    io::{Seek, SeekFrom, Write},
};

pub type Span<'a> = LocatedSpan<&'a str>;
pub type Res<'a, Output> = IResult<Span<'a>, Output, MyError<Span<'a>>>;

#[derive(Debug)]
pub enum ToplevelElement {
    EnumDecl(EnumDecl),
    StatesBlock(StatesBlock),
    SpawnBlock(SpawnBlock),
}

#[derive(Debug)]
pub struct EnumDecl {
    pub names: Vec<String>,
}

#[derive(Debug)]
pub enum StatesBlockElement {
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
pub struct StatesBlock {
    pub name: String,
    pub elements: Vec<StatesBlockElement>,
}

#[derive(Debug)]
pub struct SpawnBlock {
    pub name: String,
    pub infos: Vec<EnemySpawnInfo>,
}

#[derive(Default)]
pub struct CodegenCache {
    cache: HashMap<Vec<u8>, u64>,
}

impl CodegenCache {
    pub fn get(&mut self, codegen: &Codegen, pos: u64) -> Option<u64> {
        let cached = self.cache.get(codegen.get_code());
        if cached.is_some() {
            return cached.cloned();
        }
        self.cache.insert(codegen.get_code().into(), pos);
        None
    }
}

pub fn codegen(
    outname: &str,
    state_blocks: &[StatesBlock],
    enums: &BTreeMap<String, usize>,
    spawn_infos: &SpawnInfos,
) {
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
                } => ip += crate::STATE_BC_SIZE,
            }
        }
    }

    // // pass2: generate code
    // // calculate size of labels section
    // let mut ip = label_ptrs.keys().map(|k| k.len() as i32 + 1 + 4).sum::<i32>() + 4;
    // // 're-locate' label pointers to point after the labels section
    // println!("bc offset: {ip}");
    // label_ptrs.values_mut().for_each(|i| *i += ip);
    let mut ip = 0;

    let mut codegens = HashMap::new();
    let mut codegen_cache = CodegenCache::default();

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
                let id = *enums
                    .get(id)
                    .unwrap_or_else(|| panic!("unknown identifier {id}"))
                    as i32;
                let next_ptr = if next == "next" {
                    ip + crate::STATE_BC_SIZE
                } else {
                    let label = format!("{}::{}", state_block.name, next);
                    *label_ptrs
                        .get(&label)
                        .unwrap_or_else(|| panic!("unknown label name {label}"))
                };
                let index = states.len();
                states.push(StateBc {
                    id,
                    ticks: *ticks,
                    directional: *directional,
                    think: Function::try_from_identifier(think).unwrap_or_default(),
                    action: Function::try_from_identifier(action).unwrap_or_default(),
                    think_offs: 0,
                    action_offs: 0,
                    next: next_ptr,
                });

                codegens.insert(format!("think:{index}"), codegen_for_function_name(think));
                codegens.insert(format!("action:{index}"), codegen_for_function_name(action));
                ip += crate::STATE_BC_SIZE;
            }
        }
    }

    let mut f = std::fs::File::create(outname).expect("failed to open img file");

    // write labels and spawn info
    f.write_i32::<LittleEndian>(label_ptrs.len() as i32)
        .unwrap();
    for (name, ptr) in &label_ptrs {
        let b = name.as_bytes();
        f.write_u8(b.len() as u8).unwrap();
        let _ = f.write(b).unwrap();
        f.write_i32::<LittleEndian>(*ptr).unwrap();
    }
    spawn_infos.write(&mut f).unwrap();

    // write code
    // the bytecode blocks needs to be written first (to know their offsets), but that's ok
    // because we know how large the StateBc block will be.
    let code_start_pos = f.stream_position().unwrap();

    // first write think/action bytecode _after_ the StateBc blocks (after ip pointer)
    // fix up offset pointers in StateBc structs
    f.seek(SeekFrom::Current(ip as i64)).unwrap();
    for (i, state) in states.iter_mut().enumerate() {
        let think_name = format!("think:{i}");
        let Some(gc) = codegens.remove(&think_name) else { panic!("missing think gc")};
        let pos = f.stream_position().unwrap();
        let pos = if let Some(pos) = codegen_cache.get(&gc, pos) {
            pos
        } else {
            let _ = f.write(&gc.into_code()).unwrap();
            pos
        };
        state.think_offs = (pos - code_start_pos) as i32;
        // eprintln!("{think_name}: {pos:x}");

        let action_name = format!("action:{i}");
        let Some(gc) = codegens.remove(&action_name) else { panic!("missing action gc")};
        let pos = f.stream_position().unwrap();
        let pos = if let Some(pos) = codegen_cache.get(&gc, pos) {
            pos
        } else {
            let _ = f.write(&gc.into_code()).unwrap();
            pos
        };
        state.action_offs = (pos - code_start_pos) as i32;
        // eprintln!("{action_name}: {pos:x}");
    }
    // then write StateBc blocks _before_ bytecode
    f.seek(SeekFrom::Start(code_start_pos)).unwrap();
    for state in states {
        state.write(&mut f).unwrap();
    }
}

fn codegen_for_function_call(function: Function) -> Codegen {
    if function == Function::None {
        Codegen::default().stop()
    } else {
        Codegen::default().function_call(function).stop()
    }
}
fn codegen_for_function_name(name: &str) -> Codegen {
    // HACK: hard coded inc/dec functions for door
    if name == "IncOpen" {
        Codegen::default()
            .load_i32(0)
            .loadi_i32(1 << 10)
            .add()
            .store_i32(0)
            .stop()
    } else if name == "DecOpen" {
        Codegen::default()
            .load_i32(0)
            .loadi_i32(-(1 << 10))
            .add()
            .store_i32(0)
            .stop()
    } else {
        codegen_for_function_call(Function::try_from_identifier(name).unwrap_or_default())
    }
}

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

pub fn compile(filename: &str, outname: &str) {
    let input = std::fs::read_to_string(filename).expect("failed to read input file");
    let input = Span::new(&input);

    let (_input, toplevel_elements) =
        many1(parse_toplevel)(input).expect("failed to parse toplevel elements");
    // println!("elements: {toplevel_elements:?}");

    let mut enums = BTreeMap::new();
    let mut state_blocks = Vec::new();
    let mut spawn_infos = Vec::new();
    for tle in toplevel_elements {
        match tle {
            ToplevelElement::EnumDecl(mut enum_decl) => {
                for (i, name) in enum_decl.names.drain(..).enumerate() {
                    enums.insert(name, i + 1);
                }
            }
            ToplevelElement::StatesBlock(state_block) => state_blocks.push(state_block),
            ToplevelElement::SpawnBlock(spawn_block) => {
                for mut spawn_info in spawn_block.infos {
                    spawn_info.state = format!("{}::{}", spawn_block.name, spawn_info.state);
                    spawn_infos.push(spawn_info);
                }
            }
        }
    }

    {
        let mut enum_file = std::fs::File::create(format!("{outname}.enums")).unwrap();
        // write!(enum_file, "{enums:?}").unwrap();
        let _ = writeln!(
            enum_file,
            "const ENUM_NAMES: [(&str, i32); {}] = [",
            enums.len()
        );
        // let _ = writeln!(enum_file, "[");
        for (name, id) in enums.iter() {
            let _ = write!(enum_file, "(\"{name}\", {id}), ");
        }
        let _ = write!(enum_file, "\n];");
    }
    // std::fs::rename(from, to)
    let tmp_outname = format!("{}.tmp", outname);
    codegen(
        &tmp_outname,
        &state_blocks,
        &enums,
        &SpawnInfos { spawn_infos },
    );
    std::fs::rename(tmp_outname, outname).unwrap();
}
