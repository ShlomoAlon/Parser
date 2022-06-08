use std::iter::Flatten;
use std::rc::Rc;

use anyhow::{anyhow, Result};

fn main() {
}

type Ast<A> = A;
type ParseResult<'a, A> = Result<(&'a str, Ast<A>)>;
type Parser<A> = Rc<dyn for<'a> Fn(&'a str) -> ParseResult<'a, A>>;

trait to_literal {
    fn lit(&self) -> Parser<String>;
}

trait to_literal2 {
    fn lit(self) -> Parser<String>;
}

impl to_literal for str {
    fn lit(&self) -> Parser<String> {
        literal(self.to_string())
    }
}

impl to_literal2 for String {
    fn lit(self) -> Parser<String> {
        literal(self)
    }
}

trait Parsers<A> {
    fn many_one(self) -> Parser<Vec<A>>;
    fn map_ast<B>(self, f: impl Fn(A) -> B + 'static) -> Parser<B>;
    fn not(self) -> Parser<A>;
    fn option(self) -> Parser<Option<A>>;
    fn sequence(self, x: &mut Vec<Parser<A>>) -> Parser<Vec<A>>;
    fn choice(self, x: &mut Vec<Parser<A>>) -> Parser<A>;
    fn lift2<B: 'static, C>(self, p2: Parser<B>, f: impl Fn(A, B) -> C + 'static) -> Parser<C>;
    fn or_default(self, x: A) -> Parser<A>;
    fn bind<B>(self, f: impl Fn(A) -> Parser<B> + 'static) -> Parser<B>;

}

impl<A: 'static + Default + Clone> Parsers<A> for Parser<A> {
    fn many_one<'a>(self) -> Parser<Vec<A>> {
        many_one(self)
    }
    fn map_ast<B>(self, f: impl Fn(A) -> B + 'static) -> Parser<B> {
        map_ast(self, f)
    }
    fn not(self) -> Parser<A> {
        not(self)
    }

    fn option(self) -> Parser<Option<A>> {
        option(self)
    }

    fn sequence(self, x: &mut Vec<Parser<A>>) -> Parser<Vec<A>> {
        let mut v = vec![self];
        v.append(x);
        sequence(v)
    }

    fn choice(self, x: &mut Vec<Parser<A>>) -> Parser<A> {
        let mut v = vec![self];
        v.append(x);
        choice(v)
    }

    fn lift2<B: 'static, C>(self, p2: Parser<B>, f: impl Fn(A, B) -> C + 'static) -> Parser<C> {
        lift2(self, p2, f)
    }

    fn or_default(self, x: A) -> Parser<A> {
        or_default(self, x)
    }

    fn bind<B>(self, f: impl Fn(A) -> Parser<B> + 'static) -> Parser<B> {
        bind(self, f)
    }
}

trait vecParser<A> {
    fn choice(self) -> Parser<A>;
    fn sequence(self) -> Parser<Vec<A>>;
}

impl<A: 'static> vecParser<A> for Vec<Parser<A>> {
    fn choice(self) -> Parser<A> {
        choice(self)
    }

    fn sequence(self) -> Parser<Vec<A>> {
        sequence(self)
    }
}

pub fn literal(s: String) -> Parser<String> {
    Rc::new(move |t: &str| {
        if t.starts_with(&s) {
            let t = t.split_at(s.len());
            Ok((t.1, t.0.to_owned()))
        } else {
            Err(anyhow!("didnt work"))
        }
    })
}
fn char_predicate(f: impl Fn(char) -> bool + 'static) -> Parser<char>{
    any().bind(move |t|{
        if f(t){
            default(t)
        } else { fail(format!("unexpected char {}", t)) }
    })
}

fn bind<A: 'static, B>(p1: Parser<A>, f: impl Fn(A) -> Parser<B> + 'static) -> Parser<B>{
    Rc::new(move |t| {
        let x = p1(t)?;
        let p2 = f(x.1);
        p2(x.0)
    })}

fn float(t: &str) -> ParseResult<f64> {
    let plus_or_minus = vec!["+".lit(), "-".lit()]
        .choice()
        .or_default("+".to_string());

    let digits = (0..10)
        .map(|x| x.to_string().lit())
        .collect::<Vec<Parser<String>>>()
        .choice()
        .many_one();

    let integer = plus_or_minus.lift2(digits.clone(), |mut b, n| {
        (b + n.join("").as_str()).to_string()
    });
    let end = "."
        .lit()
        .lift2(digits.clone(), |mut b, n| {
            (b + n.join("").as_str()).to_string()
        })
        .or_default("".to_string());

    vec![integer, end]
        .sequence()
        .map_ast(|x| x.join("").parse::<f64>().unwrap())(t)
}
fn sequence<A: 'static>(s: Vec<Parser<A>>) -> Parser<Vec<A>> {
    Rc::new(move |t: &str| {
        let mut text = t;
        let mut result = vec![];
        for i in &s {
            let r = i(text)?;
            result.push(r.1);
            text = r.0;
        }
        Ok((text, result))
    })
}

fn choice<A: 'static>(s: Vec<Parser<A>>) -> Parser<A> {
    Rc::new(move |t: &str| {
        let text = t;
        let mut s = s.iter();
        loop {
            if let Ok(i) = s.next().ok_or(anyhow!("no match"))?(text) {
                return Ok(i);
            }
        }
    })
}

fn many_one<A: 'static>(s: Parser<A>) -> Parser<Vec<A>> {
    many(s, 1, 0)
}

fn many_min<A: 'static>(s: Parser<A>, min: usize) -> Parser<Vec<A>> {
    many(s, min, 0)
}

fn many<A: 'static>(s: Parser<A>, min: usize, max: usize) -> Parser<Vec<A>> {
    Rc::new(move |t: &str| {
        let mut result = vec![];
        let mut text = t;
        loop {
            match s(text) {
                Ok(i) => {
                    result.push(i.1);
                    text = i.0;
                }
                Err(_) => break,
            }
            if result.len() == max {
                break;
            }
        }
        if result.len() < min {
            Err(anyhow!("no matcht"))
        } else {
            Ok((text, result))
        }
    })
}

fn not<A: 'static + Default>(s: Parser<A>) -> Parser<A> {
    Rc::new(move |t: &str| match &s(t) {
        Ok(_) => Err(anyhow!("it succeeded when we wanted failure")),
        Err(_) => Ok((t, A::default())),
    })
}

fn map_ast<A: 'static, B>(s: Parser<A>, f: impl Fn(A) -> B + 'static) -> Parser<B> {
    Rc::new(move |t: &str| match s(t) {
        Ok(s) => Ok((s.0, f(s.1))),
        Err(_) => Err(anyhow!("didnt work")),
    })
}

fn lift2<A: 'static, B: 'static, C>(
    p1: Parser<A>,
    p2: Parser<B>,
    f: impl Fn(A, B) -> C + 'static,
) -> Parser<C> {
    Rc::new(move |t: &str| {
        let res1 = p1(t)?;
        let res2 = p2(res1.0)?;
        Ok((res2.0, f(res1.1, res2.1)))
    })
}

fn option<A: 'static>(p: Parser<A>) -> Parser<Option<A>> {
    Rc::new(move |t: &str| match p(t) {
        Ok(i) => Ok((i.0, Some(i.1))),
        Err(_) => Ok((t, None)),
    })
}

fn any() -> Parser<char> {
    Rc::new(|t: &str| {
        if t.len() == 0 {
            Err(anyhow!("String is empty"))
        } else {
            Ok((&t[1..], t.chars().nth(0).unwrap()))
        }
    })
}
fn default<A: 'static + Clone>(x: A) -> Parser<A>{
    Rc::new(move |t|{
        Ok((t, x.clone()))
    })
}

fn or_default<A: 'static + Clone>(p: Parser<A>, default: A) -> Parser<A> {
    Rc::new(move |t| Ok(p(t).unwrap_or((t, default.clone()))))
}
fn fail<A>(err: String) -> Parser<A>{
    Rc::new(move |_|{
        Err(anyhow!("{}", err))
    })
}
