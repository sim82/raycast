use std::{
    collections::BTreeMap,
    io::{Read, Write},
};

use crate::parser::{self, StateElement, Toplevel, TypedInt, Word};
use crate::util::SpanResolver;

use super::util;
use cfgrammar::Span;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use lrpar::{LexError, Lexeme, NonStreamingLexer};
// use lrlex::lrlex_mod;
// use lrpar::{lrpar_mod, NonStreamingLexer};
use state_bc::{
    codegen,
    codegen::{StatesBlock, StatesBlockElement},
    opcode::Codegen,
    Direction, EnemySpawnInfo, SpawnInfos,
};

// pub mod frontent {
pub fn compile(path: &str, outname: &str) {
    let mut input = Vec::new();
    let mut f = std::fs::File::open(path).unwrap();
    f.read_to_end(&mut input).unwrap();

    let mut input = String::from_utf8(input).unwrap();
    let lexerdef = parser::lexerdef();
    util::remove_comments(&mut input);
    let mut files = SimpleFiles::new();
    let file_id = files.add(path, input.clone());
    let lexer = lexerdef.lexer(&input);

    let (res, errs) = parser::parse(&lexer);
    for e in &errs {
        match e {
            lrpar::LexParseError::LexError(le) => {
                // println!("{}", e.pp(&lexer, &parser::token_epp))
                let s: Span = le.span();

                let diagnostic = Diagnostic::error()
                    .with_message("lex error")
                    .with_labels(vec![
                        Label::primary(file_id, s.start()..s.end()).with_message("here")
                    ])
                    .with_notes(vec![e.pp(&lexer, &parser::token_epp)]);
                let writer = StandardStream::stderr(ColorChoice::Always);
                let config = codespan_reporting::term::Config::default();

                term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
            }
            lrpar::LexParseError::ParseError(pe) => {
                let s: Span = pe.lexeme().span();

                let diagnostic = Diagnostic::error()
                    .with_message("parse error")
                    .with_labels(vec![
                        Label::primary(file_id, s.start()..s.end()).with_message("here")
                    ])
                    .with_notes(vec![e.pp(&lexer, &parser::token_epp)]);
                let writer = StandardStream::stderr(ColorChoice::Always);
                let config = codespan_reporting::term::Config::default();

                term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
            }
        }
    }
    if !errs.is_empty() {
        panic!("parse error. abort.");
    }
    // if errs.is_empty() {
    match &res {
        Some(Ok(r)) => println!("Result: {:?}", r),
        Some(Err(e)) => eprintln!("{}", e),
        _ => eprintln!("Unable to evaluate expression."),
    }
    // }
    let toplevel_elements = res.unwrap().unwrap();

    let mut enums = BTreeMap::new();
    let mut state_blocks = Vec::new();
    let mut spawn_infos = Vec::new();
    let mut function_blocks = Vec::new();

    for tle in toplevel_elements {
        match tle {
            Toplevel::States { name, elements } => {
                state_blocks.push(StatesBlock {
                    name: lexer.span_str(name).into(),
                    elements: elements
                        .iter()
                        .map(|e| states_block_to_codegen(e, &lexer))
                        .collect(),
                });
            }
            Toplevel::Spawn {
                name: name_span,
                elements,
            } => {
                let name = lexer.span_str(name_span);
                for spawn_element in elements.iter() {
                    let state = lexer.span_str(spawn_element.state);
                    let state = format!("{}::{}", name, state);
                    let bonus_item_name = lexer.span_str(spawn_element.drop);
                    // FIXME: horrible hack: put this mapping in a better location
                    fn spawn_on_death(name: &str) -> Option<i32> {
                        match name {
                            "ammo" => Some(49),
                            "silver_key" => Some(43),
                            "grofaz" => Some(224), // FIXME: abuse blinky
                            _ => None,
                        }
                    }
                    // println!("{state}");
                    if spawn_element.directional {
                        for (i, direction) in [
                            Direction::East,
                            Direction::North,
                            Direction::West,
                            Direction::South,
                        ]
                        .iter()
                        .enumerate()
                        {
                            spawn_infos.push(EnemySpawnInfo {
                                id: spawn_element.id as i32 + i as i32,
                                direction: *direction,
                                state: state.clone(),
                                spawn_on_death: spawn_on_death(&bonus_item_name),
                            })
                        }
                    } else {
                        spawn_infos.push(EnemySpawnInfo {
                            id: spawn_element.id as i32,
                            direction: Direction::South, // FIXME: not really
                            state,
                            spawn_on_death: spawn_on_death(&bonus_item_name),
                        })
                    }
                }
            }
            Toplevel::Enum { name, elements } => {
                let name = lexer.span_str(name);
                for (i, element_span) in elements.iter().enumerate() {
                    let element = lexer.span_str(*element_span);
                    // println!("{i} {name}");
                    enums.insert(format!("{}::{}", name, element), i);
                }
            }
            Toplevel::Function { name, body } => {
                function_blocks.push((name, body));
            }
        }
    }
    let mut functions = BTreeMap::new();
    for (name, body) in function_blocks {
        let name: String = lexer.span_str(name).into();

        let codegen = Codegen::default().with_annotation("source", &name);
        let codegen = emit_codegen(codegen, &body, &lexer, &enums);
        println!("'{name}'");
        functions.insert(name, codegen.stop());
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
    codegen::codegen(
        &tmp_outname,
        &state_blocks,
        &enums,
        &functions,
        &SpawnInfos { spawn_infos },
    );
    std::fs::rename(tmp_outname, outname).unwrap();
}

fn states_block_to_codegen(e: &StateElement, lexer: &dyn SpanResolver) -> StatesBlockElement {
    match e {
        StateElement::State {
            sprite: (sprite_enum, sprite_name),
            directional,
            timeout,
            think,
            action,
            next,
        } => {
            let sprite_name = lexer.get_span(*sprite_name);
            let sprite_enum = lexer.get_span(*sprite_enum);
            let id = format!("{sprite_enum}::{sprite_name}");
            let think = lexer.get_span(*think).into();
            let action = lexer.get_span(*action).into();
            let next = lexer.get_span(*next).into();

            StatesBlockElement::State {
                id,
                directional: *directional,
                ticks: *timeout as i32,
                think,
                action,
                next,
            }
        }
        StateElement::Label(label_name) => {
            // let mut full_label_name =
            //     format!("{}::{}", name, lexer.span_str(*label_name));
            // full_label_name.pop(); // FIXME: get rid of the : in some other way...
            // println!("'{full_label_name}'");
            let label_name: String = lexer.get_span(*label_name).into();
            // label_name.pop();
            StatesBlockElement::Label(label_name)
        }
    }
}

fn emit_codegen(
    mut codegen: Codegen,
    body: &[Word],
    span_resolver: &dyn SpanResolver,
    enums: &BTreeMap<String, usize>,
) -> Codegen {
    for word in body {
        codegen = match word {
            Word::Push(TypedInt::U8(v)) => codegen.loadi_u8(*v),
            Word::Push(TypedInt::I32(v)) => codegen.loadi_i32(*v),
            Word::PushStateLabel(label) => codegen.loadsl(&span_resolver.get_span(*label)[1..]), // FIXME: find better place to get rid of @
            Word::PushEnum(enum_name, name) => {
                let enum_name = span_resolver.get_span(*enum_name);
                let name = span_resolver.get_span(*name);
                let full_name = format!("{enum_name}::{name}");
                let v = enums
                    .get(&full_name)
                    .unwrap_or_else(|| panic!("could not find enum {full_name}"));
                codegen.loadi_u8(*v as u8)
            }
            Word::Trap => codegen.trap(),
            Word::Not => codegen.bin_not(),
            Word::If(body) => {
                let end_label = codegen.next_autolabel();
                emit_codegen(
                    codegen.bin_not().jrc_label(&end_label),
                    body,
                    span_resolver,
                    enums,
                )
                .label(&end_label)
            }
            Word::GoState => codegen.gostate(),
            Word::Stop => codegen.stop(),
            Word::Add => codegen.add(),
            Word::Call => codegen.call(),
            Word::WordList(body) => {
                emit_codegen(codegen, body, span_resolver, enums).loadi_u8(body.len() as u8)
            }
        }
    }
    codegen
}
