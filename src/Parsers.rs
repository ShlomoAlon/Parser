use anyhow::{anyhow, Result};
use std::ops::Deref;
use std::rc::Rc;
pub struct OkP<'a, A> {
    remaining_str: &'a str,
    ast: A,
}

impl<'a, A> OkP<'a, A> {
    fn new(remaining_str: &'a str, ast: A) -> OkP<'a, A> {
        OkP {
            remaining_str,
            ast: ast,
        }
    }
}
pub type ParseResult<'a, A> = Result<OkP<'a, A>>;

type ParserInner<A> = Rc<dyn for<'a> Fn(&'a str) -> ParseResult<'a, A>>;

pub struct Parser<A: 'static>(ParserInner<A>);

impl<A> Deref for Parser<A> {
    type Target = ParserInner<A>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<A> Parser<A> {
    fn new(p: impl for<'a> Fn(&'a str) -> ParseResult<'a, A> + 'static) -> Self {
        Parser::<A> { 0: Rc::new(p) }
    }

    pub(crate) fn map_ast<B>(self, f: impl Fn(A) -> B + 'static) -> Parser<B> {
        Parser::new(move |t: &str| match self(t) {
            Ok(s) => ok_parse(s.remaining_str, f(s.ast)),
            Err(_) => Err(anyhow!("didnt work")),
        })
    }
    pub(crate) fn lift2<B, C>(self, p2: Parser<B>, f: impl Fn(A, B) -> C + 'static) -> Parser<C> {
        Parser::new(move |t: &str| {
            let res1 = self(t)?;
            let res2 = p2(res1.remaining_str)?;
            ok_parse(res2.remaining_str, f(res1.ast, res2.ast))
        })
    }
    fn map<B>(self, f: impl Fn(ParseResult<A>) -> ParseResult<B> + 'static) -> Parser<B> {
        Parser::new(move |t| f(self(t)))
    }

    fn many(self, min: usize, max: usize) -> Parser<Vec<A>> {
        Parser::new(move |t: &str| {
            let mut result = vec![];
            let mut text = t;
            loop {
                match self(text) {
                    Ok(i) => {
                        result.push(i.ast);
                        text = i.remaining_str;
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
                ok_parse(text, result)
            }
        })
    }
    pub(crate) fn many_one(self) -> Parser<Vec<A>> {
        self.many(1, 0)
    }
    fn many_min(self, min: usize) -> Parser<Vec<A>> {
        self.many(min, 0)
    }
    fn sequence(v: Vec<Self>) -> Parser<Vec<A>> {
        Parser::new(move |t: &str| {
            let mut text = t;
            let mut result = vec![];
            for i in &v {
                let r = i(text)?;
                result.push(r.ast);
                text = r.remaining_str;
            }
            ok_parse(text, result)
        })
    }

    fn choice(s: Vec<Self>) -> Self {
        Parser::new(move |t: &str| {
            let text = t;
            let mut s = s.iter();
            loop {
                if let Ok(i) = s.next().ok_or(anyhow!("no match"))?(text) {
                    return Ok(i);
                }
            }
        })
    }

    fn option(self) -> Parser<Option<A>> {
        Parser::new(move |t: &str| match self(t) {
            Ok(i) => ok_parse(i.remaining_str, Some(i.ast)),
            Err(_) => ok_parse(t, None),
        })
    }

    fn bind<B>(self, f: impl Fn(A) -> Parser<B> + 'static) -> Parser<B> {
        Parser::new(move |t| {
            let x = self(t)?;
            let p2 = f(x.ast);
            p2(x.remaining_str)
        })
    }
    fn fail(err: String) -> Self {
        Parser::new(move |_| Err(anyhow!("didnt work")))
    }
}

impl<A: Default> Parser<A> {
    fn not(self) -> Parser<A> {
        Parser::new(move |t: &str| match &self(t) {
            Ok(_) => Err(anyhow!("it succeeded when we wanted failure")),
            Err(_) => ok_parse(t, A::default()),
        })
    }
}

impl<A: Clone> Parser<A> {
    pub(crate) fn or_default(self, default: A) -> Self {
        Parser::new(move |t| Ok(self(t).unwrap_or(OkP::new(t, default.clone()))))
    }
    fn default(x: A) -> Parser<A> {
        Parser::new(move |t| ok_parse(t, x.clone()))
    }
}
impl Parser<String> {
    pub fn literal<A: ToString>(l: A) -> Self {
        let l = l.to_string();
        Parser::new(move |t| {
            if t.starts_with(&l) {
                let (remaining_str, ast) = t.split_at(l.len());
                ok_parse(remaining_str, ast.to_string())
            } else {
                Err(anyhow!("diddnt work"))
            }
        })
    }
}

impl Parser<char> {
    fn any() -> Self {
        Parser::new(|t: &str| {
            if t.len() == 0 {
                Err(anyhow!("String is empty"))
            } else {
                ok_parse(&t[1..], t.chars().nth(0).unwrap())
            }
        })
    }
    fn char_predicate(f: impl Fn(char) -> bool + 'static) -> Self {
        Parser::any().bind(move |t| {
            if f(t) {
                Parser::default(t)
            } else {
                Parser::fail("didnt work".to_string())
            }
        })
    }
}

impl<A> Parser<Vec<A>> {
    fn join(self, parser2: Self) -> Self {
        self.lift2(parser2, |mut x, mut y| {
            x.append(&mut y);
            x
        })
    }
}
pub fn ok_parse<A>(remaining_str: &str, ast: A) -> ParseResult<A> {
    Ok(OkP::new(remaining_str, ast))
}

pub trait VecParsers<A> {
    fn choice(self) -> Parser<A>;
}

impl<A> VecParsers<A> for Vec<Parser<A>> {
    fn choice(self) -> Parser<A> {
        Parser::choice(self)
    }
}
