use std::fs::File;

use pgen::Grammar;
use pgen::Parser;

fn main() {
    let grammar = pgen::this::pgen_grammar();
    let parser = grammar.build().unwrap();

    let file = File::create("wtf/src/lib.rs").unwrap();
    parser.codegen(file).unwrap();

    parser.interpret("E", "1+1");
}
