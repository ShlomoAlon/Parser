use crate::Parsers::{Parser, VecParsers};
use std::ops::Add;

impl Parser<String> {
    pub fn then(self, p2: Self) -> Self {
        self.lift2(p2, |x, y| x.add(&y))
    }
    pub fn digit() -> Self {
        (0..10)
            .map(Parser::literal)
            .collect::<Vec<Parser<String>>>()
            .choice()
    }
    fn digits() -> Self {
        Parser::digit().many_one().flatten()
    }
    fn plus_or_minus() -> Self {
        vec![Parser::literal("+"), Parser::literal("-")]
            .choice()
            .or_default("+".to_string())
    }
    fn integer() -> Self {
        Parser::plus_or_minus().then(Parser::digits())
    }
    fn float() -> Self {
        Self::integer().then(
            Parser::literal(".")
                .then(Parser::digits())
                .or_default("".to_string()),
        )
    }
}

impl Parser<f64> {
    pub fn float() -> Self {
        Parser::<String>::float().map_ast(|x| x.parse().unwrap())
    }
}

impl Parser<i64> {
    pub fn integer() -> Self{
        Parser::<String>::integer()
            .map_ast(|x| x.parse().unwrap())
    }
}

impl Parser<Vec<String>> {
    fn flatten(self) -> Parser<String> {
        self.map_ast(|x| x.join(""))
    }
}

pub fn isAlphaNumeric() -> Parser<char>{
    Parser::char_predicate(|x| x.is_alphanumeric())
}

pub fn is_alpha() -> Parser<char>{Parser::char_predicate(|x| x.is_alphabetic())}