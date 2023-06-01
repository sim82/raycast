pub mod frontent {
    use std::{
        collections::BTreeMap,
        io::{self, BufRead, Read, Write},
        path::Path,
    };

    use self::state_bc_y::Toplevel;

    use super::util;
    use lrlex::lrlex_mod;
    use lrpar::{lrpar_mod, Lexer, NonStreamingLexer};
    use state_bc::{EnemySpawnInfo, SpawnInfos};

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
        if errs.is_empty() {
            match &res {
                Some(Ok(r)) => println!("Result: {:?}", r),
                Some(Err(e)) => eprintln!("{}", e),
                _ => eprintln!("Unable to evaluate expression."),
            }
        }
        let toplevel_elements = res.unwrap().unwrap();

        let mut enums = BTreeMap::new();
        let mut state_blocks = Vec::new();
        let mut spawn_infos = Vec::new();
        // let mut function_blocks = Vec::new();

        for tle in toplevel_elements {
            match tle {
                Toplevel::States { name, elements } => {}
                Toplevel::Spawn {
                    name: name_span,
                    elements,
                } => {
                    let name = lexer.span_str(name_span);
                    // for spawn_element in elements.iter() {
                    //     let state = lexer.span_str(spawn_element.state);
                    //     let spawn_info = EnemySpawnInfo {
                    //         id: spawn_element.id as i32,
                    //         direction: Direction::East,
                    //         state: format!("{}::{}", name, spawn_element.state),
                    //         spawn_on_death: None,
                    //     };
                    //     spawn_info.state = ;
                    //     spawn_infos.push(spawn_info);
                    // }
                }
                Toplevel::Enum(enum_decl) => {
                    for (i, name_span) in enum_decl.iter().enumerate() {
                        let name = lexer.span_str(*name_span);
                        enums.insert(name.into(), i);
                    }
                }
                Toplevel::Function { name, body } => {}
            }
        }
        let mut functions = BTreeMap::new();
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
}
pub mod util {
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
