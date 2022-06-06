use trait_set::trait_set;
use anyhow::{Result, anyhow};
fn main() {
    let f = literal("how".to_owned());
    let h = literal("ho".to_owned());
    let c = choice(vec![f.clone(), h]);
    let u = c("hohowdie").unwrap().1;
    let m = many(f)("howhowhows").unwrap().1;
    println!("{:?} {:?}", u, m)

}
type Ast<A> = A;
type ParseResult<'a, A> = Result<(&'a str, Ast<A>)>;
trait_set! {
    pub trait ThreadSafe = Send + Sync;
    pub trait Parser<'a, A> = Fn(&'a str) -> ParseResult<'a, A> + Clone
}

// fn literal<'a>(s: String) -> impl Fn(&'a str) -> ParseResult<'a, String>{
//     move |t: &'a str| {
//         if t.starts_with(&s){
//             let t = t.split_at(s.len());
//             Ok((t.0, t.1.to_owned()))
//         } else {
//             Err(anyhow!("didnt work"))
//         }
//     }
// }

fn literal<'a>(s: String) -> impl Parser<'a, String>{
    move |t: &'a str| {
        if t.starts_with(&s){
            let t = t.split_at(s.len());
            Ok((t.1, t.0.to_owned()))
        } else {
            Err(anyhow!("didnt work"))
        }
    }
}
fn sequence<'a, A: Default>(s: Vec<impl Parser<'a, A>>) -> impl Parser<'a, Vec<A>>{
    move |t: &'a str| {
        let mut text = t;
        let mut result = vec![];
        for i in &s{
            let r = i(text)?;
            result.push(r.1);
            text = r.0;
        }
        Ok((text, result))
    }
}
fn choice<'a, A>(s: Vec<impl Parser<'a, A>>) -> impl Parser<'a, A>{
    move |t: &'a str| {
        let mut text = t;
        let mut s = s.iter();
        loop {
            if let Ok(i) = s.next().ok_or(anyhow!("no match"))?(text){
                return Ok(i)
            }
        }
    }
}

fn many<'a, A>(s: impl Parser<'a, A>) -> impl Parser<'a, Vec<A>>{
    move |t: &'a str| {
        let mut result = vec![];
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
    }
}
fn not<'a, A: Default>(s: impl Parser<'a, A>) -> impl Parser<'a, A>{
    move |t: &'a str|{
        match &s(t) {
            Ok(_) => Err(anyhow!("it succeeded when we wanted failure")),
            Err(_) => Ok((t, A::default()))
        }
    }
}
fn map_ast<'a, A, B: Clone>(s: impl Parser<'a, A>, f: impl Fn(A) -> B + Clone) -> impl Parser<'a, B>{
    move |t: &'a str|{
        match s(t) {
            Ok(s) => Ok((s.0, f(s.1))),
            Err(_) => Err(anyhow!("didnt work"))
        }
    }
}


