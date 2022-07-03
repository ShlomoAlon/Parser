use crate::parsefloat::{isAlphaNumeric, is_alpha, white_space};
use crate::parsemal::Maltype::{Bool, List, Nil, Num, Str, Symbol};
use crate::Parser;
use crate::Parsers::VecParsers;
use anyhow::Result;
use lazy_static::lazy_static;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Maltype {
    Str(String),
    Symbol(String),
    Num(f64),
    Nil,
    Bool(bool),
    List(Rc<Vec<Maltype>>),
    // Func(Rc<dyn Fn(Vec<Maltype>) -> Result<Maltype>>),
}
pub fn parse_expr() -> Parser<Maltype> {
    Parser::choice(vec![atom(), parse_list()])
}
pub fn parse_nil() -> Parser<Maltype> {
    Parser::literal("nil").discard_then_parse(Parser::default(Nil))
}
fn parse_bool() -> Parser<Maltype> {
    let f = Parser::literal("false").discard_then_parse(Parser::default(Bool(false)));
    let t = Parser::literal("true").discard_then_parse(Parser::default(Bool(true)));
    vec![f, t].choice()
}
fn parse_symbol() -> Parser<Maltype> {
    is_alpha()
        .map_ast(|x| x.to_string())
        .then(
            isAlphaNumeric()
                .map_ast(|x| x.to_string())
                .many_min(0)
                .map_ast(|x| x.join("")),
        )
        .map_ast(|x| Symbol(x))
}
fn parse_float() -> Parser<Maltype> {
    Parser::float().map_ast(|x| Num(x))
}
pub fn parse_str() -> Parser<Maltype> {
    let double_quote = Parser::literal("\"");
    let escape_char = Parser::literal("\\")
        .discard_then_parse(Parser::any())
        .map_ast(|x| match x {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            other => other,
        });
    let non_escape_char = Parser::char_predicate(|x| x != '"');
    let str_char = vec![escape_char, non_escape_char].choice();
    double_quote
        .clone()
        .discard_then_parse(str_char.map_ast(|x| x.to_string()).many_min(0))
        .map_ast(|x| Str(x.join("")))
        .parse_then_discard(double_quote)
}
fn parse_list() -> Parser<Maltype> {
    let rob_pike = parse_expr().sep_by(white_space().many_min(1)).map_ast(|x| List(Rc::new(x)));

    Parser::literal("(")
        .discard_then_parse(white_space().many_min(0))
        .discard_then_parse(rob_pike)
        .parse_then_discard(white_space().many_min(0))
        .parse_then_discard(Parser::literal(")"))
}
fn atom() -> Parser<Maltype> {
    Parser::choice(vec![parse_nil(), parse_bool(), parse_symbol(), parse_str(), parse_float()])
}
