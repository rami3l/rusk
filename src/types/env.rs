use super::exp::Exp;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The Scheme Environment Model.
#[derive(Clone)]
pub struct Env {
    pub data: HashMap<String, Exp>,
    pub outer: Option<RcRefCell<Env>>,
}

impl Env {
    pub fn from_outer(outer: Option<RcRefCell<Env>>) -> Env {
        Env {
            data: HashMap::new(),
            outer,
        }
    }

    pub fn lookup(&self, symbol: &Exp) -> Option<Exp> {
        // find the definition of a symbol
        match symbol {
            Exp::Symbol(s) => match self.data.get(s) {
                Some(def) => Some(def.clone()),
                None => match &self.outer {
                    Some(outer) => outer.borrow().lookup(symbol),
                    None => None,
                },
            },
            _ => None,
        }
    }
}

/// A pointer type for easier Environment Operations
pub type RcRefCell<T> = Rc<RefCell<T>>;

pub fn make_env_ptr(env: Env) -> RcRefCell<Env> {
    Rc::new(RefCell::new(env))
}
