use std::io::Cursor;

use wtf::Parser;

fn main() {
    let mut parser = Parser::new("1+2*3");
    println!("{:?}", parser.parse_E());
}
