use crate::types::*;
use regex::Regex;
use rustyline;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};

// * Parsing, refactored

lazy_static! {
    pub static ref TOKENIZER: Regex =
        Regex::new(r#"\s*(,@|[('`,)]|"(?:[\\].|[^\\"])*"|;.*|[^\s('"`,;)]*)(.*)"#).unwrap();
}

fn atom(token: &str) -> Exp {
    match token.parse::<f64>() {
        Ok(num) => Exp::Number(num),
        Err(_) => Exp::Symbol(token.to_string()),
    }
}

fn desugar(exp: Exp) -> Result<Exp, ScmErr> {
    // Handle syntax sugar forms.

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
                        match require_len(&list, 3) {
                            Ok(()) => (),
                            Err(e) => return Err(e),
                        };
                        let f: Exp; // Symbol
                        let args: Exp; // List
                        let body: Vec<Exp>;
                        match &list[1] {
                            Exp::List(f_args) => {
                                match require_len(f_args, 1) {
                                    Ok(()) => (),
                                    Err(e) => return Err(e),
                                };
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
                                        [Exp::Symbol("lambda".to_string()), args]
                                            .iter()
                                            .map(|x| x.clone())
                                            .chain(body.into_iter())
                                            .collect();
                                    let res: Vec<Exp> = [
                                        Exp::Symbol("define".to_string()),
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

                    "lambda" => {
                        // (lambda args body+) => (lambda args (begin body+))
                        match require_len(&list, 3) {
                            Ok(()) => (),
                            Err(e) => return Err(e),
                        };
                        let args: Exp = list[1].clone(); // Listargs = list[1].clone();
                        let body: Vec<Exp> = list.iter().skip(2).map(|x| x.clone()).collect();
                        let definition = match list.len() {
                            0 | 1 | 2 => unreachable!(),
                            3 => list[2].clone(),
                            _ => {
                                let begin_body: Vec<Exp> = [Exp::Symbol("begin".to_string())]
                                    .iter()
                                    .map(|x| x.clone())
                                    .chain(body.into_iter())
                                    .collect();
                                Exp::List(begin_body)
                            }
                        };
                        let lambda_args_definition: Vec<Exp> = [
                            Exp::Symbol("lambda".to_string()),
                            args,
                            desugar(definition).unwrap(),
                        ]
                        .iter()
                        .map(|x| x.clone())
                        .collect();
                        Ok(Exp::List(lambda_args_definition))
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

pub trait InPort {
    // * An input port/stream based on the implementation on http://norvig.com/lispy2.html

    fn readline(&mut self) -> Option<Result<String, Box<dyn Error>>>;

    fn next_token(&mut self) -> Option<Result<String, Box<dyn Error>>>;

    fn read_ahead(&mut self, token: &str) -> Result<Exp, ScmErr> {
        match token {
            "(" => {
                let mut l: Vec<Exp> = Vec::new();
                loop {
                    let next = self.next_token();
                    match next {
                        Some(Ok(t)) => match t.as_ref() {
                            ")" => return Ok(Exp::List(l)),
                            _ => l.push(self.read_ahead(&t).unwrap()),
                        },
                        Some(Err(e)) => return Err(ScmErr::from(&format!("{}", e))),
                        None => return Err(ScmErr::from("parser: Unexpected EOF")),
                    }
                }
            }
            ")" => Err(ScmErr::from("parser: Extra \")\" found")),
            // ! quotes unimplemented
            _ => Ok(atom(token)),
        }
    }

    fn read_exp(&mut self, token: Option<Result<String, Box<dyn Error>>>) -> Result<Exp, ScmErr> {
        // Read an Exp starting from given token.
        match token {
            Some(Ok(t)) => match self.read_ahead(&t) {
                Ok(exp) => desugar(exp),
                Err(e) => Err(e),
            },
            Some(Err(e)) => return Err(ScmErr::from(&format!("{}", e))),
            None => Ok(Exp::Empty),
        }
    }

    fn read_next_exp(&mut self) -> Result<Exp, ScmErr> {
        // Read an Exp starting from next token.
        let next = self.next_token();
        self.read_exp(next)
    }
}

pub struct InFile {
    pub file_str: String,
    line: String,
    reader: BufReader<File>,
}

impl InFile {
    pub fn new(file_str: &str) -> InFile {
        InFile {
            file_str: file_str.to_string(),
            line: String::new(),
            reader: {
                let file = OpenOptions::new()
                    .read(true)
                    .write(false)
                    .open(file_str)
                    .unwrap();
                BufReader::new(file)
            },
        }
    }
}

impl InPort for InFile {
    fn readline(&mut self) -> Option<Result<String, Box<dyn Error>>> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => Some(Ok(line)),
            Err(e) => Some(Err(Box::new(e))),
        }
    }

    fn next_token(&mut self) -> Option<Result<String, Box<dyn Error>>> {
        loop {
            if &self.line == "" {
                self.line = match self.readline() {
                    Some(Ok(line)) => line,
                    None => String::new(),
                    _ => unreachable!(),
                };
            }
            if &self.line == "" {
                return None;
            } else {
                let next = TOKENIZER.captures_iter(&self.line).next();
                let (token, rest) = match next {
                    Some(cap) => (String::from(&cap[1]), String::from(&cap[2])),
                    None => unreachable!(),
                };
                self.line = rest;
                match token.chars().nth(0) {
                    Some(';') | None => (),
                    _ => return Some(Ok(token.to_string())),
                };
            }
        }
    }
}

pub struct Input {
    line: String,
    editor: rustyline::Editor<()>,
    // * The following is for a better REPL experience
    // count: u64,  // the input expression count
    ended: bool, // indicates if the expression has ended when a line begins
}

impl Input {
    pub fn new() -> Input {
        Input {
            line: String::new(),
            editor: rustyline::Editor::<()>::new(),
            // count: 0,
            ended: true,
        }
    }
}

impl InPort for Input {
    fn readline(&mut self) -> Option<Result<String, Box<dyn Error>>> {
        let prompt = if self.ended {
            // self.count += 1;
            // format!("#;{}> ", self.count)
            ">> ".to_string()
        } else {
            ".. ".to_string()
        };
        // self.count += 1;
        // self.editor.readline(&format!("#;{}> ", self.count))
        match self.editor.readline(&prompt) {
            Ok(s) => Some(Ok(s)),
            Err(e) => Some(Err(Box::new(e))),
        }
    }

    fn next_token(&mut self) -> Option<Result<String, Box<dyn Error>>> {
        loop {
            if &self.line == "" {
                self.line = match self.readline() {
                    Some(Ok(line)) => line,
                    None => String::new(),
                    Some(Err(e)) => return Some(Err(e)),
                };
            }
            if &self.line == "" {
                return None;
            } else {
                let next = TOKENIZER.captures_iter(&self.line).next();
                let (token, rest) = match next {
                    Some(cap) => (String::from(&cap[1]), String::from(&cap[2])),
                    None => unreachable!(),
                };
                self.line = rest;
                match token.chars().nth(0) {
                    Some(';') | None => (),
                    _ => return Some(Ok(token.to_string())),
                };
            }
        }
    }

    fn read_exp(&mut self, token: Option<Result<String, Box<dyn Error>>>) -> Result<Exp, ScmErr> {
        // Read an Exp starting from given token, plus modifying the self.ended flag.
        self.ended = false;
        let res = match token {
            Some(Ok(t)) => match self.read_ahead(&t) {
                Ok(exp) => desugar(exp),
                Err(e) => Err(e),
            },
            Some(Err(e)) => return Err(ScmErr::from(&format!("{}", e))),
            None => Ok(Exp::Empty),
        };
        self.ended = true;
        res
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
