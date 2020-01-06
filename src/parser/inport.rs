use super::{atom, desugar, TOKENIZER};
use crate::types::*;
use rustyline;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};

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
                            _ => match self.read_ahead(&t) {
                                Ok(exp) => l.push(exp),
                                Err(e) => return Err(e),
                            },
                        },
                        Some(Err(e)) => return Err(ScmErr::from(&format!("{}", e))),
                        None => return Err(ScmErr::from("parser: Unexpected EOF")),
                    }
                }
            }
            ")" => Err(ScmErr::from("parser: Extra \")\" found")),
            // TODO: quote
            _ => Ok(atom(token)),
        }
    }

    /// Read an Exp starting from given token.
    fn read_exp(&mut self, token: Option<Result<String, Box<dyn Error>>>) -> Result<Exp, ScmErr> {
        match token {
            Some(Ok(t)) => match self.read_ahead(&t) {
                Ok(exp) => desugar(exp),
                Err(e) => Err(e),
            },
            Some(Err(e)) => Err(ScmErr::from(&format!("{}", e))),
            None => Ok(Exp::Empty),
        }
    }

    /// Read an Exp starting from next token.
    fn read_next_exp(&mut self) -> Result<Exp, ScmErr> {
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
            file_str: file_str.into(),
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
                    _ => return Some(Ok(token.into())),
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
                    _ => return Some(Ok(token.into())),
                };
            }
        }
    }

    /// Read an Exp starting from given token.
    /// Modify the self.ended flag at the same time.
    fn read_exp(&mut self, token: Option<Result<String, Box<dyn Error>>>) -> Result<Exp, ScmErr> {
        self.ended = false;
        let res = match token {
            Some(Ok(t)) => match self.read_ahead(&t) {
                Ok(exp) => desugar(exp),
                Err(e) => Err(e),
            },
            Some(Err(e)) => Err(ScmErr::from(&format!("{}", e))),
            None => Ok(Exp::Empty),
        };
        self.ended = true;
        res
    }
}
