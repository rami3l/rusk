use super::{desugar, InPort};
use crate::types::*;
use rustyline;
use std::cell::RefCell;
use std::error::Error;

pub struct Input {
    line: Option<String>,
    editor: RefCell<rustyline::Editor<()>>,
    // * The following is for a better REPL experience
    // count: u64,  // the input expression count
    ended: bool, // indicates if the expression has ended when a line begins
}

impl Input {
    pub fn new() -> Self {
        Self {
            line: Some("".into()),
            editor: RefCell::new(rustyline::Editor::<()>::new()),
            // count: 0,
            ended: true,
        }
    }
}

impl InPort for Input {
    fn line(&self) -> Option<String> {
        self.line.clone()
    }

    fn set_line(&mut self, new_line: Option<String>) {
        self.line = new_line;
    }

    fn read_line(&self) -> Result<Option<String>, Box<dyn Error>> {
        let prompt = if self.ended {
            // self.count += 1;
            // format!("#;{}> ", self.count)
            ">> ".to_string()
        } else {
            ".. ".to_string()
        };
        // self.count += 1;
        // self.editor.readline(&format!("#;{}> ", self.count))
        match self.editor.borrow_mut().readline(&prompt) {
            Ok(s) => Ok(Some(s)),
            Err(e) => Err(Box::new(e)),
        }
    }

    /// Read an Exp starting from given token.
    /// Modify the self.ended flag at the same time.
    fn read_exp(&mut self, token: Result<Option<String>, Box<dyn Error>>) -> Result<Exp, ScmErr> {
        self.ended = false;
        let res = match token {
            Ok(Some(t)) => match self.read_ahead(&t) {
                // * Enable/Disable desugaring
                Ok(exp) => desugar(exp),
                // Ok(exp) => Ok(exp),
                Err(e) => Err(e),
            },
            Ok(None) => Ok(Exp::Empty),
            Err(e) => Err(ScmErr::from(&format!("{}", e))),
        };
        self.ended = true;
        res
    }
}
