use super::{atom, desugar, TOKENIZER};
use crate::types::*;
use std::error::Error;

pub mod infile;
pub mod input;

pub use infile::InFile;
pub use input::Input;

pub trait InPort {
    // * An input port/stream based on the implementation on http://norvig.com/lispy2.html

    fn line(&self) -> Option<String>;

    fn set_line(&mut self, new_line: Option<String>);

    fn read_line(&self) -> Result<Option<String>, Box<dyn Error>>;
    // Got a line: OK(Some(string))
    // No new lines: Ok(None)
    // Readline Error: Err(e)

    fn next_token(&mut self) -> Result<Option<String>, Box<dyn Error>> {
        loop {
            if self.line() == None {
                return Ok(None);
            } else if self.line().unwrap().is_empty() {
                self.set_line(match self.read_line() {
                    Ok(x) => x,
                    Err(e) => return Err(e),
                });
                continue;
            } else {
                let line = self.line().unwrap();
                let next = TOKENIZER.captures_iter(&line).next();
                let (token, rest): (String, String) = {
                    let cap = next.unwrap();
                    (cap[1].into(), cap[2].into())
                };
                self.set_line(Some(rest));
                match token.chars().nth(0) {
                    Some(';') | None => (),
                    _ => return Ok(Some(token.into())),
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
                        Ok(Some(t)) => match t.as_ref() {
                            ")" => return Ok(Exp::List(l)),
                            _ => l.push(self.read_ahead(&t)?),
                        },
                        Ok(None) => return Err(ScmErr::from("parser: Unexpected EOF")),
                        Err(e) => return Err(ScmErr::from(&format!("{}", e))),
                    }
                }
            }
            ")" => Err(ScmErr::from("parser: Extra \")\" found")),
            // TODO: quote
            _ => Ok(atom(token)),
        }
    }

    /// Read an Exp starting from the given token.
    fn read_exp(&mut self, token: Result<Option<String>, Box<dyn Error>>) -> Result<Exp, ScmErr> {
        match token {
            Ok(Some(t)) => match self.read_ahead(&t) {
                // * Enable/Disable desugaring
                Ok(exp) => desugar(exp),
                // Ok(exp) => Ok(exp),
                Err(e) => Err(e),
            },
            Ok(None) => Ok(Exp::Empty),
            Err(e) => Err(ScmErr::from(&format!("{}", e))),
        }
    }

    /// Read an Exp starting from the next token.
    fn read_next_exp(&mut self) -> Result<Exp, ScmErr> {
        let next = self.next_token();
        self.read_exp(next)
    }
}
