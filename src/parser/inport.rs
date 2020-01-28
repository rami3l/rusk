use super::{atom, desugar, TOKENIZER};
use crate::types::*;
use std::error::Error;

pub mod infile;
pub mod input;

pub use infile::InFile;
pub use input::Input;

pub trait InPort {
    // * An input port/stream based on the implementation on http://norvig.com/lispy2.html

    fn line(&self) -> String;

    fn set_line(&mut self, new_line: &str);

    fn read_line(&self) -> Option<Result<String, Box<dyn Error>>>;

    fn next_token(&mut self) -> Option<Result<String, Box<dyn Error>>> {
        loop {
            if self.line().is_empty() {
                self.set_line(&match self.read_line() {
                    Some(Ok(line)) => line,
                    None => String::new(),
                    Some(Err(e)) => return Some(Err(e)),
                });
            }
            if self.line().is_empty() {
                return None;
            } else {
                let line = self.line();
                let next = TOKENIZER.captures_iter(&line).next();
                let (token, rest) = match next {
                    Some(cap) => (String::from(&cap[1]), String::from(&cap[2])),
                    None => unreachable!(),
                };
                self.set_line(&rest);
                match token.chars().nth(0) {
                    Some(';') | None => (),
                    _ => return Some(Ok(token.into())),
                };
            }
        }
    }

    fn read_ahead(&mut self, token: &str) -> Result<Exp, ScmErr> {
        match token {
            "(" => {
                let mut l: Vec<Exp> = Vec::new();
                loop {
                    let next = self.next_token();
                    match next {
                        Some(Ok(t)) => match t.as_ref() {
                            ")" => return Ok(Exp::List(l)),
                            _ => l.push(self.read_ahead(&t)?),
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
                // * Enable/Disable desugaring
                Ok(exp) => desugar(exp),
                // Ok(exp) => Ok(exp),
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
