use crate::Parsers::Parser;

mod Parsers;
mod parsefloat;
mod parsemal;

fn main() {
    let x = Parser::float()("-0.4").unwrap().ast;
    println!("{}", x)


}


