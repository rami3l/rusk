use super::{Env, ScmErr};
use std::fmt;

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

impl fmt::Display for Exp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = match self {
            Exp::Bool(b) => format!("{}", b),
            Exp::Symbol(s) => format!("'{}", s),
            Exp::Number(n) => format!("{}", n),
            Exp::List(l) => format!("{:?}", l),
            Exp::Closure(_) => "<Closure>".into(),
            Exp::Primitive(_) => "<Primitive>".into(),
            Exp::Empty => "()".into(),
        };
        write!(f, "{}", res)
    }
}

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

/// A Closure is a user-defined function.
/// It has a function body and a captured environment.
#[derive(Clone)]
pub struct ScmClosure {
    pub body: Box<Exp>,
    pub env: Env,
}
