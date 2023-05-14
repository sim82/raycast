use crate::{
    ms::{endian::WriteExt, Writable},
    opcode::Codegen,
    SpawnInfos, StateBc,
};
use std::{
    collections::{BTreeMap, HashMap},
    io::{Cursor, Write},
};

use super::ast::{StatesBlock, StatesBlockElement};
#[derive(Default)]
struct BytecodeOutput {
    code: Vec<u8>,
    start_ptr: i32,
}

impl BytecodeOutput {
    pub fn new(start_ptr: i32) -> Self {
        Self {
            start_ptr,
            code: vec![0u8; start_ptr as usize],
        }
    }
    pub fn append_codegen(&mut self, codegen: Codegen) -> i32 {
        let mut code = codegen.finalize();
        if self.code.len() >= code.len() {
            // compression / size optimization:
            // search for subsequence match in previously generated code
            if let Some(pos) = self
                .code
                .windows(code.len())
                .position(|window| window == code)
            {
                return pos as i32;
            }
        }
        let pos = self.code.len() as i32;
        self.code.append(&mut code);
        pos
    }

    fn write_states(&mut self, states: &[StateBc]) {
        let mut f = Cursor::new(&mut self.code[0..(self.start_ptr as usize)]);
        for state in states {
            state.write(&mut f).expect(
                "failed to write StateBc block. Probably pre-calculated output size is wrong.",
            );
        }
    }
}

pub fn codegen(
    outname: &str,
    state_blocks: &[StatesBlock],
    enums: &BTreeMap<String, usize>,
    functions: &BTreeMap<String, Codegen>,
    spawn_infos: &SpawnInfos,
) {
    let _ = std::fs::rename(outname, format!("{outname}.bak")); // don't care if it does not work

    let mut states = Vec::new();
    let mut label_ptrs = BTreeMap::new(); // keep them sorted in the output file
    let mut ps_label_ptrs = Vec::new();

    // pass 1: resolve label offsets
    let mut ip = 0;
    for state_block in state_blocks {
        let mut label_ptrs2 = HashMap::new();
        for element in &state_block.elements {
            match element {
                StatesBlockElement::Label(name) => {
                    label_ptrs.insert(format!("{}::{}", state_block.name, name), ip);
                    label_ptrs2.insert(name.clone(), ip);
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
        ps_label_ptrs.push(label_ptrs2);
    }

    // // pass2: generate code
    // // calculate size of labels section
    // let mut ip = label_ptrs.keys().map(|k| k.len() as i32 + 1 + 4).sum::<i32>() + 4;
    // // 're-locate' label pointers to point after the labels section
    // println!("bc offset: {ip}");
    // label_ptrs.values_mut().for_each(|i| *i += ip);
    let mut ip = 0;

    let mut codegens = Vec::new();
    for (state_block, label_ptrs2) in state_blocks.iter().zip(ps_label_ptrs) {
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
                    think_offs: 0,
                    action_offs: 0,
                    next: next_ptr,
                });

                codegens.push((
                    format!("think:{index}"),
                    codegen_for_function_name(think, functions, &label_ptrs2),
                ));
                codegens.push((
                    format!("action:{index}"),
                    codegen_for_function_name(action, functions, &label_ptrs2),
                ));
                ip += crate::STATE_BC_SIZE;
            }
        }
    }

    let mut f = std::fs::File::create(outname).expect("failed to open img file");

    // write labels and spawn info
    f.writei32(label_ptrs.len() as i32).unwrap();
    for (name, ptr) in &label_ptrs {
        let b = name.as_bytes();
        f.writeu8(b.len() as u8).unwrap();
        let _ = f.write(b).unwrap();
        f.writei32(*ptr).unwrap();
    }
    spawn_infos.write(&mut f).unwrap();

    // luxury feature: sort bytecode blocks by descending size. This way the bytecode
    // compression should be approximately ideal (there might be better ordering to also
    // leverage matches across different blocks, but yeah well...)
    codegens.sort_unstable_by_key(|(_, codegen)| -(codegen.len() as i64));
    let mut bytecode_output = BytecodeOutput::new(ip);
    let mut bc_pos = HashMap::new();
    let mut map_f =
        std::fs::File::create(format!("{outname}.map")).expect("failed to open map file");
    for (name, codegen) in codegens {
        let pos = bytecode_output.append_codegen(codegen.clone());
        bc_pos.insert(name, pos);
        writeln!(
            map_f,
            "{} - {} {}",
            pos,
            pos + codegen.len() as i32,
            codegen
                .get_annotation("source")
                .expect("missing annotation 'source'"),
        )
        .unwrap();
    }
    for (i, state) in states.iter_mut().enumerate() {
        let think_name = format!("think:{i}");
        state.think_offs = *bc_pos
            .get(&think_name)
            .expect("missing bc offset for {think_name}");

        let action_name = format!("action:{i}");
        state.action_offs = *bc_pos
            .get(&action_name)
            .expect("missing bc offset for {action_name}");
    }
    bytecode_output.write_states(&states);
    let _ = f.write(&bytecode_output.code).unwrap();
}

fn codegen_for_function_name<'a>(
    name: &str,
    functions: &'a BTreeMap<String, Codegen>,
    label_ptrs: &HashMap<String, i32>,
) -> Codegen {
    functions
        .get(name)
        .unwrap_or_else(|| panic!("unknown function {name}"))
        .clone()
        .with_state_label_ptrs(label_ptrs)
}
