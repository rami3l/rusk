use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::rc::Rc;

pub type RcRefCellBox<T> = Rc<RefCell<Box<T>>>;

// * Types

#[derive(Clone)]
pub enum Exp {
    Bool(bool),
    Symbol(String),
    Number(f64),         // ! int unimplemented
    List(VecDeque<Exp>), // also used as AST
    Closure(ScmClosure),
    Primitive(fn(&[Exp]) -> Result<Exp, ScmErr>),
    Empty,
}

impl fmt::Debug for Exp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            Exp::Bool(b) => format!("{}", b),
            Exp::Symbol(s) => s.to_string(),
            Exp::Number(n) => format!("{}", n),
            Exp::List(l) => format!("{:?}", l),
            Exp::Closure(_) => "<Closure>".to_string(),
            Exp::Primitive(_) => "<Primitive>".to_string(),
            Exp::Empty => String::new(),
        };
        write!(f, "{}", res)
    }
}

#[derive(Clone)]
pub struct Env {
    pub data: HashMap<String, Exp>,
    pub outer: Option<RcRefCellBox<Env>>,
}

impl Env {
    pub fn from_outer(outer: Option<RcRefCellBox<Env>>) -> Env {
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

#[derive(Clone)]
pub struct ScmClosure {
    pub body: Box<Exp>,
    pub env: Env,
}

pub enum ScmErr {
    Reason(String),
}

impl ScmErr {
    pub fn from(reason: &str) -> ScmErr {
        ScmErr::Reason(String::from(reason))
    }
}

impl fmt::Debug for ScmErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self {
            ScmErr::Reason(res) => res.clone(),
        };
        write!(f, "{}", reason)
    }
}
