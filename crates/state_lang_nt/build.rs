#![deny(rust_2018_idioms)]
use cfgrammar::yacc::YaccKind;
use lrlex::{self, CTLexerBuilder};

fn main() {
    // Since we're using both lrlex and lrpar, we use lrlex's `lrpar_config` convenience function
    // that makes it easy to a) create a lexer and parser and b) link them together.
    CTLexerBuilder::new()
        .rust_edition(lrlex::RustEdition::Rust2021)
        .lrpar_config(|ctp| {
            ctp.yacckind(YaccKind::Grmtools)
                .rust_edition(lrpar::RustEdition::Rust2021)
                .grammar_in_src_dir("state_bc.y")
                .unwrap()
        })
        .lexer_in_src_dir("state_bc.l")
        .unwrap()
        .build()
        .unwrap();
}
