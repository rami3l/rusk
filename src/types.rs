use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

pub type RcRefCellBox<T> = Rc<RefCell<Box<T>>>;

// * Types

#[derive(Clone)]
pub enum Exp {
    Bool(bool),
    Symbol(String),
    Number(f64),    // ! int unimplemented
    List(Vec<Exp>), // also used as AST
    Closure(ScmClosure),
    Primitive(fn(&[Exp]) -> Result<Exp, ScmErr>),
    Empty,
}

// TODO: implement fmt::Display for Exp for better user experience

impl fmt::Debug for Exp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            Exp::Bool(b) => format!("{}", b),
            Exp::Symbol(s) => format!("'{}", s),
            Exp::Number(n) => format!("{}", n),
            Exp::List(l) => format!("{:?}", l),
            Exp::Closure(_) => "<Closure>".into(),
            Exp::Primitive(_) => "<Primitive>".into(),
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

pub struct ScmErr {
    reason: String,
}

impl ScmErr {
    pub fn from(reason: &str) -> ScmErr {
        ScmErr {
            reason: String::from(reason),
        }
    }
}

impl fmt::Debug for ScmErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.reason)
    }
}
