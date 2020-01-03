use std::io::Cursor;

use wtf::Parser;

fn main() {
    let parser = Parser::new();
    println!("{:?}", parser.parse_Expr(Cursor::new("1+2*3")));
}
