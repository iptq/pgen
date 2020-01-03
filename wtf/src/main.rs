use std::io::Cursor;

use wtf::Parser;

fn main() {
    let parser = Parser::new();
    println!("{:?}", parser.parse_E("1+2*3"));
}
