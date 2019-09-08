use crate::types::*;
use regex::Regex;
use rustyline;
use std::collections::VecDeque;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};

/*

// * Parsing

fn tokenize(str_exp: &str) -> VecDeque<String> {
    let res: VecDeque<_> = str_exp
        .replace("(", " ( ")
        .replace(")", " ) ")
        .split_whitespace()
        .map(String::from)
        .collect();
    res
}



fn gen_ast(tokens: &mut VecDeque<String>) -> Result<Exp, ScmErr> {
    if tokens.is_empty() {
        return Err(ScmErr::from("gen_ast: Unexpected EOF"));
    }

    let head = tokens.pop_front().unwrap();
    match head.as_ref() {
        // if we have a list ahead of us, we return that list
        "(" => {
            let mut res = VecDeque::new();
            loop {
                match tokens.get(0) {
                    Some(t) => match t.as_ref() {
                        ")" => break,
                        _ => match gen_ast(tokens) {
                            Ok(Exp::List(l)) => res.push_back(Exp::List(l)),
                            Ok(Exp::Symbol(s)) => res.push_back(Exp::Symbol(s)),
                            Ok(Exp::Number(f)) => res.push_back(Exp::Number(f)),
                            // recursion: deal with the tail of the list
                            // ! Attention: we are appending sub-expressions (including atoms) to the result
                            // Todo: refactor this function
                            Err(e) => return Err(e),
                            _ => {
                                return Err(ScmErr::from(
                                    "gen_ast: Expected a deque of Symbol, Number or List",
                                ))
                            }
                        },
                    },
                    None => return Err(ScmErr::from("gen_ast: Mismatched parens")),
                }
            }
            tokens.pop_front(); // pop off ")"
            Ok(Exp::List(res))
        }
        ")" => Err(ScmErr::from("gen_ast: Extra \")\" found")),

        // if the head is a single atom, we return just the head
        _ => Ok(atom(&head)),
    }
}

pub fn parse(str_exp: &str) -> Result<Exp, ScmErr> {
    let mut ast = tokenize(str_exp);
    gen_ast(&mut ast)
}

*/

// * Parsing, alternative

fn atom(token: &str) -> Exp {
    match token.parse::<f64>() {
        Ok(num) => Exp::Number(num),
        Err(_) => Exp::Symbol(token.to_string()),
    }
}

lazy_static! {
    pub static ref TOKENIZER: Regex =
        Regex::new(r#"\s*(,@|[('`,)]|"(?:[\\].|[^\\"])*"|;.*|[^\s('"`,;)]*)(.*)"#).unwrap();
}

pub trait InPort {
    // * An input port/stream based on the implementation on http://norvig.com/lispy2.html

    fn readline(&mut self) -> Option<Result<String, Box<dyn Error>>>;

    fn next_token(&mut self) -> Option<Result<String, Box<dyn Error>>>;

    fn read_ahead(&mut self, token: &str) -> Result<Exp, ScmErr> {
        match token {
            "(" => {
                let mut l: VecDeque<Exp> = VecDeque::new();
                loop {
                    let next = self.next_token();
                    match next {
                        Some(Ok(t)) => match t.as_ref() {
                            ")" => return Ok(Exp::List(l)),
                            _ => l.push_back(self.read_ahead(&t).unwrap()),
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

    fn read(&mut self) -> Result<Exp, ScmErr> {
        let next = self.next_token();
        match next {
            Some(Ok(t)) => self.read_ahead(&t),
            Some(Err(e)) => return Err(ScmErr::from(&format!("{}", e))),
            None => Ok(Exp::Empty),
        }
    }
}

pub struct InFile {
    file_str: String,
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
    count: u64,
    editor: rustyline::Editor<()>,
}

impl Input {
    pub fn new() -> Input {
        Input {
            line: String::new(),
            count: 0,
            editor: rustyline::Editor::<()>::new(),
        }
    }
}

impl InPort for Input {
    fn readline(&mut self) -> Option<Result<String, Box<dyn Error>>> {
        self.count += 1;
        match self.editor.readline(&format!("#;{}> ", self.count)) {
            Ok(s) => {
                if s.is_empty() {
                    None
                } else {
                    Some(Ok(s))
                }
            }
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

/*

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let left = "(+ 1 2)";
        let right: VecDeque<String> = vec!["(", "+", "1", "2", ")"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        assert_eq!(tokenize(left), right)
    }

    #[test]
    fn test_parse() {
        let left = "(+ 1 2)";
        let right: VecDeque<Exp> = vec![
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
