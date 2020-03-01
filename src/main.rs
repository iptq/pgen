use std::fs::File;

use pgen::Grammar;
use pgen::Parser;

fn main() {
    let grammar = pgen::this::pgen_grammar();
    let parser = grammar.build().unwrap();

    parser.interpret("E", "1+2*3");

    let file = File::create("wtf/src/lib.rs").unwrap();
    parser.codegen(file);
}
