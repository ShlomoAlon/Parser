mod Parsers;


fn main() {}

// fn float(t: &str) -> ParseResult<f64> {
//     let plus_or_minus = vec!["+".lit(), "-".lit()]
//         .choice()
//         .or_default("+".to_string());
//
//     let digits = (0..10)
//         .map(|x| x.to_string().lit())
//         .collect::<Vec<Parsers<String>>>()
//         .choice()
//         .many_one();
//
//     let integer = plus_or_minus.lift2(digits.clone(), |mut b, n| {
//         (b + n.join("").as_str()).to_string()
//     });
//     let end = "."
//         .lit()
//         .lift2(digits.clone(), |mut b, n| {
//             (b + n.join("").as_str()).to_string()
//         })
//         .or_default("".to_string());
//
//     vec![integer, end]
//         .sequence()
//         .map_ast(|x| x.join("").parse::<f64>().unwrap())(t)
// }
