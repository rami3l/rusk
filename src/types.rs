use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

// * Types

/// The Scheme Expression type.
#[derive(Clone)]
pub enum Exp {
    /// A Bool (#t/#f).
    Bool(bool),
    /// A Symbol.
    Symbol(String),
    /// A Number. Actually a f64.
    Number(f64), // ! int unimplemented
    /// A List. Also used as AST.
    List(Vec<Exp>),
    /// A user-defined function.
    Closure(ScmClosure),
    /// A Primitive function. Provided by the Prelude.
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

/// A Closure is a user-defined function.
/// It has a function body and a captured environment.
#[derive(Clone)]
pub struct ScmClosure {
    pub body: Box<Exp>,
    pub env: Env,
}

#[derive(Debug)]
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

impl fmt::Display for ScmErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.reason)
    }
}

impl std::error::Error for ScmErr {}
