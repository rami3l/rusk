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
    pub fn from_outer(outer: Option<RcRefCell<Env>>) -> Self {
        Self {
            data: HashMap::new(),
            outer,
        }
    }

    /// Find the definition of a symbol.
    pub fn lookup(&self, symbol: &Exp) -> Option<Exp> {
        match symbol {
            Exp::Symbol(s) => match self.data.get(s) {
                Some(def) => Some(def.clone()),
                None => self.outer.as_ref().and_then(|o| o.borrow().lookup(symbol)),
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
