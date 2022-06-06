use trait_set::trait_set;
use anyhow::{Result, anyhow};
fn main() {
    let f = literal("ho".to_owned());
    let h = literal("how".to_owned());
    let c = sequence(vec![f, h]);
    let u = c("hohowdie").unwrap().1;
    println!("{:?} ", u)

}
type Ast<A> = A;
type ParseResult<'a, A> = Result<(&'a str, Ast<A>)>;
trait_set! {
    pub trait ThreadSafe = Send + Sync;
    pub trait Parser<'a, A> = Fn(&'a str) -> ParseResult<'a, A>
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
fn sequence<'a, A>(s: Vec<impl Parser<'a, A>>) -> impl Parser<'a, Vec<A>>{
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

