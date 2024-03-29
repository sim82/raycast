use self::{
    ast::ToplevelElement,
    parser::{parse_toplevel, util::Span},
};
use crate::{opcode::Codegen, SpawnInfos};
use nom::multi::many1;
use std::{collections::BTreeMap, io::Write};

pub mod ast;
pub mod codegen;
pub mod parser;

pub fn compile(filename: &str, outname: &str) {
    let input = std::fs::read_to_string(filename).expect("failed to read input file");
    let input = Span::new(&input);

    let (_input, toplevel_elements) =
        many1(parse_toplevel)(input).expect("failed to parse toplevel elements");
    // println!("elements: {toplevel_elements:?}");

    let mut enums = BTreeMap::new();
    let mut state_blocks = Vec::new();
    let mut spawn_infos = Vec::new();
    let mut function_blocks = Vec::new();
    for tle in toplevel_elements {
        match tle {
            ToplevelElement::EnumDecl(mut enum_decl) => {
                for (i, name) in enum_decl.names.drain(..).enumerate() {
                    enums.insert(name, i);
                }
            }
            ToplevelElement::StatesBlock(state_block) => state_blocks.push(state_block),
            ToplevelElement::SpawnBlock(spawn_block) => {
                for mut spawn_info in spawn_block.infos {
                    spawn_info.state = format!("{}::{}", spawn_block.name, spawn_info.state);
                    spawn_infos.push(spawn_info);
                }
            }
            ToplevelElement::FunctionBlock(function_block) => function_blocks.push(function_block),
        }
    }

    let mut functions = BTreeMap::new();
    for function_block in function_blocks {
        let mut codegen = Codegen::default().with_annotation("source", &function_block.name);
        for element in function_block.elements {
            codegen = match element {
                ast::FunctionBlockElement::Label(label) => codegen.label(&label),
                ast::FunctionBlockElement::LoadiI32 { value } => codegen.loadi_i32(value),
                ast::FunctionBlockElement::LoadiU8Enum { name } => {
                    let v = enums
                        .get(&name)
                        .unwrap_or_else(|| panic!("could not find enum {name}"));
                    codegen.loadi_u8(*v as u8)
                }
                ast::FunctionBlockElement::LoadiU8 { value } => codegen.loadi_u8(value),
                ast::FunctionBlockElement::Trap => codegen.trap(),
                ast::FunctionBlockElement::Add => codegen.add(),
                ast::FunctionBlockElement::FunctionCall => codegen.call(),
                ast::FunctionBlockElement::Ceq => codegen.ceq(),
                ast::FunctionBlockElement::Not => codegen.bin_not(),
                ast::FunctionBlockElement::Jrc { label } => codegen.jrc_label(&label),
                ast::FunctionBlockElement::Stop => codegen.stop(),
                ast::FunctionBlockElement::GoState => codegen.gostate(),
                ast::FunctionBlockElement::LoadStateLabel { label } => codegen.loadsl(&label),
            };
        }
        functions.insert(function_block.name.clone(), codegen.stop());
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
