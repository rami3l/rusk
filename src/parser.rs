use crate::types::*;
use regex::Regex;
// use std::fs::{File, OpenOptions};
// use std::io::{BufRead, BufReader};

mod inport;
pub use inport::{InFile, InPort, Input};

// * Parsing, refactored

lazy_static! {
    pub static ref TOKENIZER: Regex =
        Regex::new(r#"\s*(,@|[('`,)]|"(?:[\\].|[^\\"])*"|;.*|[^\s('"`,;)]*)(.*)"#).unwrap();
}

pub fn atom(token: &str) -> Exp {
    match token.parse::<f64>() {
        Ok(num) => Exp::Number(num),
        Err(_) => Exp::Symbol(token.into()),
    }
}

/// Handles syntax sugar forms.
// ! This is EXTREMELY DIRTY.
// TODO: refactor this function in a more elegant way.
pub fn desugar(exp: Exp) -> Result<Exp, ScmErr> {
    fn require_len(list: &Vec<Exp>, min_len: usize) -> Result<(), ScmErr> {
        let len = list.len();
        if len < min_len {
            Err(ScmErr::from(&format!(
                "desugar: too few arguments ({}/{})",
                len, min_len
            )))
        } else {
            Ok(())
        }
    }

    match exp.clone() {
        Exp::List(list) => {
            // println!("Sugar debug: {:?}", list);
            match list.get(0) {
                Some(Exp::Symbol(s)) => match s.as_ref() {
                    "define" => {
                        // (define (f . args) body+) => (define f (lambda args body+))
                        require_len(&list, 3)?;
                        let f: Exp; // Symbol
                        let args: Exp; // List
                        let body: Vec<Exp>;
                        match &list[1] {
                            Exp::List(f_args) => {
                                require_len(f_args, 1)?;
                                f = match f_args[0].clone() {
                                    Exp::Symbol(s) => Exp::Symbol(s),
                                    _ => {
                                        return Err(ScmErr::from(
                                            "desugar: can only define a Symbol",
                                        ))
                                    }
                                };
                                args = {
                                    let args_list: Vec<Exp> =
                                        f_args.iter().skip(1).map(|x| x.clone()).collect();
                                    Exp::List(args_list)
                                };
                                body = list.iter().skip(2).map(|x| x.clone()).collect();
                                desugar(Exp::List({
                                    let lambda_args_body: Vec<Exp> =
                                        [Exp::Symbol("lambda".into()), args]
                                            .iter()
                                            .map(|x| x.clone())
                                            .chain(body.into_iter())
                                            .collect();
                                    let res: Vec<Exp> = [
                                        Exp::Symbol("define".into()),
                                        f,
                                        Exp::List(lambda_args_body),
                                    ]
                                    .iter()
                                    .map(|x| x.clone())
                                    .collect();
                                    // println!("Sugar debug: {:?}", res);
                                    res
                                }))
                            }
                            _ => {
                                let res: Vec<Exp> =
                                    list.iter().map(|x| desugar(x.clone()).unwrap()).collect();
                                // println!("Sugar debug: {:?}", res);
                                Ok(Exp::List(res))
                            }
                        }
                    }

                    _ => Ok(exp),
                },

                _ => {
                    let res: Vec<Exp> = list.iter().map(|x| desugar(x.clone()).unwrap()).collect();
                    Ok(Exp::List(res))
                }
            }
        }
        _ => Ok(exp), // a non-list cannot be expanded.
    }
}

/*

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let left = "(+ 1 2)";
        let right: Vec<String> = vec!["(", "+", "1", "2", ")"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        assert_eq!(tokenize(left), right)
    }

    #[test]
    fn test_parse() {
        let left = "(+ 1 2)";
        let right: Vec<Exp> = vec![
            Exp::Symbol("+".to_string()),
            Exp::Number(1 as f64),
            Exp::Number(2 as f64),
        ]
        .iter()
        .map(|x| x.clone())
        .collect();
        let left = match parse(left) {
            Ok(Exp::List(l)) => l,
            _ => panic!("should parse to a list"),
        };
        for (l, r) in left.iter().zip(right.iter()) {
            match (l, r) {
                (Exp::Symbol(ls), Exp::Symbol(rs)) => assert_eq!(ls, rs),
                (Exp::Number(lf), Exp::Number(rf)) => assert_eq!(lf, rf),
                _ => panic!("should be Symbol or Number"),
            }
        }
    }
}

*/
