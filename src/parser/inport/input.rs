use super::{desugar, InPort};
use crate::types::*;
use rustyline;
use std::cell::RefCell;
use std::error::Error;

pub struct Input {
    line: String,
    editor: RefCell<rustyline::Editor<()>>,
    // * The following is for a better REPL experience
    // count: u64,  // the input expression count
    ended: bool, // indicates if the expression has ended when a line begins
}

impl Input {
    pub fn new() -> Self {
        Self {
            line: String::new(),
            editor: RefCell::new(rustyline::Editor::<()>::new()),
            // count: 0,
            ended: true,
        }
    }
}

impl InPort for Input {
    fn line(&self) -> String {
        self.line.clone()
    }

    fn set_line(&mut self, new_line: &str) {
        self.line = new_line.into();
    }

    fn readline(&self) -> Option<Result<String, Box<dyn Error>>> {
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
            Ok(s) => Some(Ok(s)),
            Err(e) => Some(Err(Box::new(e))),
        }
    }

    /// Read an Exp starting from given token.
    /// Modify the self.ended flag at the same time.
    fn read_exp(&mut self, token: Option<Result<String, Box<dyn Error>>>) -> Result<Exp, ScmErr> {
        self.ended = false;
        let res = match token {
            Some(Ok(t)) => match self.read_ahead(&t) {
                // * Enable/Disable desugaring
                Ok(exp) => desugar(exp),
                // Ok(exp) => Ok(exp),
                Err(e) => Err(e),
            },
            Some(Err(e)) => Err(ScmErr::from(&format!("{}", e))),
            None => Ok(Exp::Empty),
        };
        self.ended = true;
        res
    }
}
