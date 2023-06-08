pub mod frontent {
    use std::{
        collections::BTreeMap,
        io::{self, BufRead, Read, Write},
        path::Path,
    };

    use crate::util::SpanResolver;

    use self::state_bc_y::{Toplevel, TypedInt, Word};

    use super::util;
    use lrlex::lrlex_mod;
    use lrpar::{lrpar_mod, Lexer, NonStreamingLexer};
    use state_bc::{
        compiler::ast::{FunctionBlock, StatesBlock, StatesBlockElement},
        opcode::Codegen,
        Direction, EnemySpawnInfo, SpawnInfos,
    };

    lrlex_mod!("state_bc.l");
    lrpar_mod!("state_bc.y");
    // }
    // pub mod frontent {
    pub fn compile(path: &str, outname: &str) {
        let lexerdef = state_bc_l::lexerdef();
        let mut input = Vec::new();
        let mut f = std::fs::File::open(path).unwrap();
        f.read_to_end(&mut input).unwrap();

        let mut input = String::from_utf8(input).unwrap();
        util::remove_comments(&mut input);
        let lexer = lexerdef.lexer(&input);

        let (res, errs) = state_bc_y::parse(&lexer);
        for e in &errs {
            println!("{}", e.pp(&lexer, &state_bc_y::token_epp));
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
                    let mut out_elements = Vec::new();
                    let name = lexer.span_str(name).into();
                    for e in elements.iter() {
                        match e {
                            state_bc_y::StateElement::State {
                                sprite: (sprite_enum, sprite_name),
                                directional,
                                timeout,
                                think,
                                action,
                                next,
                            } => {
                                let sprite_name = lexer.span_str(*sprite_name);
                                let sprite_enum = lexer.span_str(*sprite_enum);
                                let id = format!("{sprite_enum}::{sprite_name}");
                                let think = lexer.span_str(*think).into();
                                let action = lexer.span_str(*action).into();
                                let next = lexer.span_str(*next).into();

                                out_elements.push(StatesBlockElement::State {
                                    id,
                                    directional: *directional,
                                    ticks: *timeout as i32,
                                    think,
                                    action,
                                    next,
                                })
                            }
                            state_bc_y::StateElement::Label(label_name) => {
                                // let mut full_label_name =
                                //     format!("{}::{}", name, lexer.span_str(*label_name));
                                // full_label_name.pop(); // FIXME: get rid of the : in some other way...
                                // println!("'{full_label_name}'");
                                let mut label_name: String = lexer.span_str(*label_name).into();
                                // label_name.pop();
                                out_elements.push(StatesBlockElement::Label(label_name));
                            }
                        }
                    }
                    state_blocks.push(StatesBlock {
                        name,
                        elements: out_elements,
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

            let mut codegen = Codegen::default().with_annotation("source", &name);
            let codegen = emit_code(codegen, &body, &lexer, &enums);
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
        state_bc::compiler::codegen::codegen(
            &tmp_outname,
            &state_blocks,
            &enums,
            &functions,
            &SpawnInfos { spawn_infos },
        );
        std::fs::rename(tmp_outname, outname).unwrap();
    }

    fn emit_code(
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
                    emit_code(
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
                    emit_code(codegen, body, span_resolver, enums).loadi_u8(body.len() as u8)
                }
            }
        }
        codegen
    }
}
pub mod util {
    use cfgrammar::Span;
    use lrlex::{DefaultLexerTypes, LRNonStreamingLexer};
    use lrpar::NonStreamingLexer;

    pub fn remove_comments(input: &mut String) {
        let mut comments = Vec::new();
        let mut in_comment = false;
        let mut potential_start = None;
        let mut start = None;
        for (i, c) in input.char_indices() {
            if c == '/' {
                if start.is_none() && potential_start.is_some() {
                    start = potential_start;
                } else {
                    potential_start = Some(i);
                }
            } else if c == '\n' {
                if let Some(start) = &start {
                    comments.push((*start, i));
                }
                potential_start = None;
                start = None;
            } else {
                potential_start = None;
            }
        }
        comments.reverse();
        for (start, end) in comments {
            input.replace_range(start..end, "");
        }
    }
    pub trait SpanResolver {
        fn get_span(&self, span: Span) -> &str;
    }
    impl SpanResolver for LRNonStreamingLexer<'_, '_, DefaultLexerTypes> {
        fn get_span(&self, span: Span) -> &str {
            self.span_str(span)
        }
    }
}
#[test]
fn test_remove_comments() {
    let mut input: String = r"
        bla
        bla2 // comment
        //comment 2
        bla
    "
    .into();
    util::remove_comments(&mut input);
    println!("{input}");
}
mod test {
    enum Test {
        A,
        B,
        C,
    }
    fn test() {
        let a = Test::A;
    }
}
