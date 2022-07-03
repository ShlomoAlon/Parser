use crate::parsemal::{parse_expr, parse_str};
use crate::Parsers::Parser;

mod Parsers;
mod parsefloat;
mod parsemal;

fn main() {
    let x = Parser::float()("-0.4").unwrap().ast;
    println!("{}", x);
    let x = parse_str()("\"h\\el\\nlo\"");
    println!("{:?}", x.unwrap().ast);
    let x = parse_expr()("( + 1 1)");
    println!("{:?}", x.unwrap().ast)

}
