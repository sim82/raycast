use std::{
    collections::{BTreeMap, HashSet},
    io::{Read, Write},
};

use crate::parser::{self, FunctionRef, StateElement, Toplevel, TypedInt, Word};
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

enum DiagnosticDesc {
    ParseError {
        label: String,
        span: Span,
        note: String,
    },
    LexError {
        label: String,
        span: Span,
        note: String,
    },
    UndefinedReference {
        label: String,
        span: Span,
        identifier: String,
    },
    // UndefinedReferenceQualified {
    //     label: String,
    //     span: Span,
    //     identifier_ns: Span,
    //     identifier: Span,
    // },
}
struct ErrorReporter {
    files: SimpleFiles<String, String>,
    file_id: usize,
    known_identifier: HashSet<String>,
}
impl ErrorReporter {
    pub fn new(filename: &str, input: &str) -> Self {
        let mut files = SimpleFiles::new();
        let file_id = files.add(filename.into(), input.into());
        ErrorReporter {
            files,
            file_id,
            known_identifier: HashSet::new(),
        }
    }
    fn report_error(&self, message: &str, label: &str, span: Span, note: &str) {
        let label = Label::primary(self.file_id, span.start()..span.end()).with_message(label);
        let diagnostic = Diagnostic::error()
            .with_message(message)
            .with_labels(vec![label])
            .with_notes(vec![note.into()]);
        let writer = StandardStream::stderr(ColorChoice::Always);
        let config = codespan_reporting::term::Config::default();

        term::emit(&mut writer.lock(), &config, &self.files, &diagnostic).unwrap();
    }
    fn report_diagnostic(&self, diagnostic: &DiagnosticDesc) {
        match diagnostic {
            DiagnosticDesc::ParseError { label, span, note } => {
                self.report_error("parse error", label, *span, note)
            }
            DiagnosticDesc::LexError { label, span, note } => {
                self.report_error("lex error", label, *span, note)
            }
            DiagnosticDesc::UndefinedReference {
                label,
                span,
                identifier,
            } => self.report_error(
                "undefined reference",
                &format!("undefined: {identifier}"),
                *span,
                &self.suggest_identifier(identifier),
            ),
            // DiagnosticDesc::UndefinedReferenceQualified {
            //     label,
            //     span,
            //     identifier_ns,
            //     identifier,
            // } => self.report_error(
            //     "undefined reference",
            //     &format!("undefined: {identifier}"),
            //     *span,
            //     &self.suggest_identifier(identifier),
            // ),
        }
    }
    fn suggest_identifier(&self, identifier: &str) -> String {
        if let Some(m) = self.get_fuzzy_match(identifier) {
            format!("did you mean '{m}'")
        } else {
            "no similar known identifier".into()
        }
    }
    fn get_fuzzy_match(&self, s: &str) -> Option<String> {
        let candidates: Vec<(&str, usize)> = self
            .known_identifier
            .iter()
            .enumerate()
            .map(|(i, s)| (s.as_str(), i))
            .collect();
        fuzzy_match::fuzzy_match(s, candidates.clone()).map(|i| candidates[i].0.to_string())
    }

    fn add_identifiers<'a>(&mut self, keys: impl IntoIterator<Item = &'a str>) {
        for identifier in keys.into_iter() {
            self.known_identifier.insert(identifier.into());
        }
    }
    fn check_identifier(&self, identifier: &str, span: Span) {
        if !self.known_identifier.contains(identifier) {
            self.report_diagnostic(&DiagnosticDesc::UndefinedReference {
                label: "".into(),
                span,
                identifier: identifier.into(),
            })
        }
    }
}
trait EnumResolver {
    fn resolve(
        &self,
        enum_name: Span,
        name: Span,
        span_resolver: &dyn SpanResolver,
    ) -> Option<usize>;
    fn resolve_unqual(
        &self,
        name: Span,
        span_resolver: &dyn SpanResolver,
        error_reporter: &ErrorReporter,
    ) -> Option<usize>;
}
struct EnumResolverFlat {
    enums: BTreeMap<String, usize>,
}
impl EnumResolver for EnumResolverFlat {
    fn resolve(
        &self,
        enum_name: Span,
        name: Span,
        span_resolver: &dyn SpanResolver,
    ) -> Option<usize> {
        let enum_name = span_resolver.get_span(enum_name);
        let name = span_resolver.get_span(name);
        let full_name = format!("{enum_name}::{name}");
        self.enums.get(&full_name).cloned()
    }

    fn resolve_unqual(
        &self,
        name: Span,
        span_resolver: &dyn SpanResolver,
        error_reporter: &ErrorReporter,
    ) -> Option<usize> {
        // flat resolver can never resolve an unqualified enum
        None
    }
}
struct EnumResolverUsing {
    enums: BTreeMap<String, usize>,
    uses: Vec<Span>,
}
impl EnumResolver for EnumResolverUsing {
    fn resolve(
        &self,
        enum_name: Span,
        name: Span,
        span_resolver: &dyn SpanResolver,
    ) -> Option<usize> {
        let enum_name = span_resolver.get_span(enum_name);
        let name = span_resolver.get_span(name);
        let full_name = format!("{enum_name}::{name}");
        self.enums.get(&full_name).cloned()
    }

    fn resolve_unqual(
        &self,
        name_span: Span,
        span_resolver: &dyn SpanResolver,
        error_reporter: &ErrorReporter,
    ) -> Option<usize> {
        let name = span_resolver.get_span(name_span);
        let mut resolved = None;
        let mut resolved_name = None;
        // let Some(uses) = &self.uses else { return None };
        for enum_name in &self.uses {
            let enum_name = span_resolver.get_span(*enum_name);
            let full_name = format!("{enum_name}::{name}");
            let cand = self.enums.get(&full_name);
            if let (Some(match1_name), Some(match2)) = (&resolved_name, cand) {
                // TODO: better error message
                error_reporter.report_error(
                    "ambiguous unqualified enum",
                    &format!("{match1_name} vs {full_name}"),
                    name_span,
                    "meep",
                );
                return None;
            }
            if cand.is_some() {
                // println!("resolved: {full_name}");
                resolved = cand.cloned();
                resolved_name = Some(full_name);
            }
        }
        // println!("resolved: {resolved:?}");
        resolved
    }
}
// pub mod frontent {
pub fn compile(path: &str, outname: &str) {
    let mut input = Vec::new();
    let mut f = std::fs::File::open(path).unwrap();
    f.read_to_end(&mut input).unwrap();

    let mut input = String::from_utf8(input).unwrap();
    let lexerdef = parser::lexerdef();
    util::remove_comments(&mut input);
    // let mut files = SimpleFiles::new();
    // let file_id = files.add(path, input.clone());
    let mut error_reporter = ErrorReporter::new(path, &input);
    error_reporter.add_identifiers(["None"]);
    let lexer = lexerdef.lexer(&input);

    let (res, errs) = parser::parse(&lexer);
    for e in &errs {
        match e {
            lrpar::LexParseError::LexError(le) => {
                // println!("{}", e.pp(&lexer, &parser::token_epp))
                let s: Span = le.span();
                error_reporter.report_diagnostic(&DiagnosticDesc::LexError {
                    label: "here".into(),
                    span: s,
                    note: e.pp(&lexer, &parser::token_epp),
                });
            }
            lrpar::LexParseError::ParseError(pe) => {
                let s: Span = pe.lexeme().span();

                error_reporter.report_diagnostic(&DiagnosticDesc::ParseError {
                    label: "here".into(),
                    span: s,
                    note: e.pp(&lexer, &parser::token_epp),
                });
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
    // pass 1: extract function / enum declarations
    for tle in &toplevel_elements {
        match tle {
            Toplevel::Enum { name, elements } => {
                let name = lexer.span_str(*name);
                for (i, element_span) in elements.iter().enumerate() {
                    let element = lexer.span_str(*element_span);
                    // println!("{i} {name}");
                    enums.insert(format!("{}::{}", name, element), i);
                }
            }
            Toplevel::Function { decl, body } => {
                let name: String = lexer.span_str(decl.name).into();
                function_blocks.push((name, Some(decl.clone()), body.clone()));
            }
            _ => (),
        }
    }
    error_reporter.add_identifiers(enums.keys().map(|e| e.as_str()));
    error_reporter.add_identifiers(function_blocks.iter().map(|(name, _, _)| name.as_str()));
    // pass2: process states / spawn blocks
    let mut inline_function_count = 0;
    for tle in toplevel_elements {
        match tle {
            Toplevel::States { name, elements } => {
                let mut elements2 = Vec::new();
                for e in &elements {
                    let x = match e {
                        StateElement::State {
                            sprite: (sprite_enum, sprite_name),
                            directional,
                            timeout,
                            think,
                            action,
                            next,
                        } => {
                            let id = format!(
                                "{}::{}",
                                lexer.get_span(*sprite_enum),
                                lexer.get_span(*sprite_name)
                            );
                            let think = match think {
                                FunctionRef::Name(name) => {
                                    let s = lexer.get_span(*name);
                                    error_reporter.check_identifier(&s, *name);
                                    s.into()
                                }
                                FunctionRef::Inline(body) => {
                                    let name = format!("InlineThink{}", inline_function_count);
                                    inline_function_count += 1;
                                    function_blocks.push((name.clone(), None, body.clone()));
                                    name
                                }
                            };
                            let action = match action {
                                FunctionRef::Name(name) => {
                                    let s = lexer.get_span(*name);
                                    error_reporter.check_identifier(&s, *name);
                                    s.into()
                                }
                                FunctionRef::Inline(body) => {
                                    let name = format!("InlineAction{}", inline_function_count);
                                    inline_function_count += 1;
                                    function_blocks.push((name.clone(), None, body.clone()));
                                    name
                                }
                            };
                            let next = lexer.get_span(*next).into();
                            error_reporter.check_identifier(
                                &id,
                                Span::new(sprite_enum.start(), sprite_name.end()),
                            );

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
                            let label_name: String = lexer.get_span(*label_name).into();
                            StatesBlockElement::Label(label_name)
                        }
                    };
                    elements2.push(x);
                }
                state_blocks.push(StatesBlock {
                    name: lexer.span_str(name).into(),
                    elements: elements2,
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
            _ => (),
        }
    }
    let mut functions = BTreeMap::new();
    for (name, decl, body) in function_blocks {
        let enum_resolver = if let Some(using) = decl.map(|decl| decl.using.clone()) {
            Box::new(EnumResolverUsing {
                enums: enums.clone(),
                uses: using,
            }) as Box<dyn EnumResolver>
        } else {
            Box::new(EnumResolverFlat {
                enums: enums.clone(),
            })
        };
        let codegen = Codegen::default().with_annotation("source", &name);
        let codegen = emit_codegen(codegen, &body, &lexer, &*enum_resolver, &error_reporter);
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

fn emit_codegen(
    mut codegen: Codegen,
    body: &[Word],
    span_resolver: &dyn SpanResolver,
    // enums: &BTreeMap<String, usize>,
    enum_resolver: &dyn EnumResolver,
    error_reporter: &ErrorReporter,
) -> Codegen {
    for word in body {
        codegen = match word {
            Word::Push(TypedInt::U8(v)) => codegen.loadi_u8(*v),
            Word::Push(TypedInt::I32(v)) => codegen.loadi_i32(*v),
            Word::PushStateLabel(label) => codegen.loadsl(&span_resolver.get_span(*label)[1..]), // FIXME: find better place to get rid of @
            Word::PushEnum(enum_name, name) => {
                // let full_name = format!(
                //     "{}::{}",
                //     span_resolver.get_span(*enum_name),
                //     span_resolver.get_span(*name)
                // );
                // if let Some(v) = enums.get(&full_name) {
                if let Some(v) = enum_resolver.resolve(*enum_name, *name, span_resolver) {
                    codegen.loadi_u8(v as u8)
                } else {
                    // FIXME: crappy
                    let full_name = format!(
                        "{}::{}",
                        span_resolver.get_span(*enum_name),
                        span_resolver.get_span(*name)
                    );
                    error_reporter.report_diagnostic(&DiagnosticDesc::UndefinedReference {
                        label: "here".into(),
                        span: Span::new(enum_name.start(), name.end()),
                        identifier: full_name.clone(),
                    });
                    panic!();
                }
                // .unwrap_or_else(|| panic!("could not find enum {full_name}"));
            }
            Word::PushEnumUnqual(name) => {
                if let Some(v) = enum_resolver.resolve_unqual(*name, span_resolver, error_reporter)
                {
                    codegen.loadi_u8(v as u8)
                } else {
                    error_reporter.report_diagnostic(&DiagnosticDesc::UndefinedReference {
                        label: "here".into(),
                        span: *name,
                        identifier: span_resolver.get_span(*name).into(),
                    });
                    panic!();
                }
            }
            Word::Trap => codegen.trap(),
            Word::Not => codegen.bin_not(),
            Word::If(body) => {
                let end_label = codegen.next_autolabel();
                emit_codegen(
                    codegen.bin_not().jrc_label(&end_label),
                    body,
                    span_resolver,
                    enum_resolver,
                    error_reporter,
                )
                .label(&end_label)
            }
            Word::GoState => codegen.gostate(),
            Word::Stop => codegen.stop(),
            Word::Add => codegen.add(),
            Word::Call => codegen.call(),
            Word::WordList(body) => {
                emit_codegen(codegen, body, span_resolver, enum_resolver, error_reporter)
                    .loadi_u8(body.len() as u8)
            }
        }
    }
    codegen
}
