use std::rc::Rc;
use std::sync::Arc;
use trait_set::trait_set;
use anyhow::{Result, anyhow};
use dyn_clone::DynClone;

fn main() {
    let plus  = literal("+".to_owned());
    let minus =  literal("-".to_owned());
    let plus_or_minus = choice(vec![plus, minus]);
    let plus_or_minus = option(plus_or_minus);
    let plus_or_minus = map_ast(plus_or_minus, |x| x.unwrap_or("".to_string()));
    let digit = choice(
        (0..10).map(|x| literal(x.to_string())).collect());
    let digits = many_one(digit);
    let integer = lift2(plus_or_minus, digits, |mut b, n|{
        b.push_str(n.join("").as_str())
    });
    let p = integer("+00012345   ").unwrap();

    println!("{:?}", p);
}
type Ast<A> = A;
type ParseResult<'a, A> = Result<(&'a str, Ast<A>)>;
type Parser<A> = Rc<dyn for <'a> Fn(&'a str) -> ParseResult<'a, A>>;

pub fn literal(s: String) -> Parser<String>{
    Rc::new(move |t: & str| {
        if t.starts_with(&s){
            let t = t.split_at(s.len());
            Ok((t.1, t.0.to_owned()))
        } else {
            Err(anyhow!("didnt work"))
        }
    })
}
fn sequence<A: 'static>(s: Vec<Parser<A>>) -> Parser<Vec<A>>{
    Rc::new(move |t: & str| {
        let mut text = t;
        let mut result = vec![];
        for i in &s{
            let r = i(text)?;
            result.push(r.1);
            text = r.0;
        }
        Ok((text, result))
    })
}
fn choice<A: 'static>(s: Vec<Parser<A>>) -> Parser<A>{
    Rc::new(move |t: & str| {
        let text = t;
        let mut s = s.iter();
        loop {
            if let Ok(i) = s.next().ok_or(anyhow!("no match"))?(text){
                return Ok(i)
            }
        }
    })
}

fn many_one<A: 'static>(s: Parser<A>) -> Parser<Vec<A>>{ Rc::new(move |t: & str| { let mut result = vec![];
        let mut text = t;
        loop {
            match s(text) {
                Ok(i) => {
                    result.push(i.1);
                    text = i.0;
                }
                Err(_) => break
            }
        }
        if result.len() == 0{
            Err(anyhow!("no matcht"))
        } else {
            Ok((text, result))
        }
    })
}

fn many<A: 'static>(s: Parser<A>, min: usize, max: usize) -> Parser<Vec<A>>{
    Rc::new(move |t: & str| { let mut result = vec![];
    let mut text = t;
    loop {
        match s(text) {
            Ok(i) => {
                result.push(i.1);
                text = i.0;
            }
            Err(_) => break
        }
        if result.len() == max{
            break
        }
    }
    if result.len() < min{
        Err(anyhow!("no matcht"))
    } else {
        Ok((text, result))
    }
})
}
fn not<A: 'static + Default>(s: Parser<A>) -> Parser<A>{
    Rc::new(move |t: & str|{
        match &s(t) {
            Ok(_) => Err(anyhow!("it succeeded when we wanted failure")),
            Err(_) => Ok((t, A::default()))
        }
    })
}
fn map_ast<A: 'static, B>(s: Parser<A>, f: impl Fn(A) -> B + 'static) -> Parser<B>{
    Rc::new(move |t: & str|{
        match s(t) {
            Ok(s) => Ok((s.0, f(s.1))),
            Err(_) => Err(anyhow!("didnt work"))
        }
    })
}
fn lift2<A: 'static, B: 'static, C>(p1: Parser<A>, p2: Parser<B>, f: impl Fn(A, B) -> C + 'static) ->
Parser<C>{
    Rc::new(move |t: & str| {
        let res1 = p1(t)?;
        let res2 = p2(res1.0)?;
        Ok((res2.0, f(res1.1, res2.1)))
    })
}

fn option<A: 'static>(p: Parser<A>) -> Parser<Option<A>>{
    Rc::new(move |t: & str| {
        match p(t) {
            Ok(i) => Ok((i.0, Some(i.1))),
            Err(_)=> Ok((t, None))
        }
    })
}
