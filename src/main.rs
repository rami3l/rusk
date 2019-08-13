use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
// use std::cell::RefCell;

// Types
#[derive(Clone)]
enum Exp {
    Bool(bool),
    Symbol(String),
    Number(f64),         // ! int unimplemented
    List(VecDeque<Exp>), // also used as AST
    Lambda(ScmLambda),
    Primitive(fn(&[Exp]) -> Option<Exp>),
}

#[derive(Clone)]
struct ScmLambda {
    params: Rc<Exp>,
    body: Rc<Exp>,
}

#[derive(Clone)]
struct Env<'a> {
    data: HashMap<String, Exp>,
    outer: Option<&'a Env<'a>>,
}

impl<'a> Env<'a> {
    fn new(outer: Option<&'a Env<'a>>) -> Env {
        Env {
            data: HashMap::new(),
            outer,
        }
    }
    fn find(&self, symbol: &Exp) -> Option<&Env> {
        // find the innermost Env where symbol appears
        match symbol {
            Exp::Symbol(s) => match self.data.get(s) {
                Some(_) => Some(&self),
                None => match self.outer {
                    Some(outer) => outer.find(symbol),
                    None => None,
                },
            },
            _ => None,
        }
    }

    fn lookup(&self, symbol: &Exp) -> Option<Exp> {
        match symbol {
            Exp::Symbol(s) => match self.data.get(s) {
                Some(exp) => Some(exp.clone()),
                None => None,
            },
            _ => None,
        }
    }
}

struct ScmClosure<'a> {
    body: Rc<Exp>,
    env: Env<'a>,
}

#[derive(Debug)]
enum ScmErr {
    Reason(String),
}

impl ScmErr {
    fn from(reason: &str) -> ScmErr {
        ScmErr::Reason(String::from(reason))
    }
}

/*
// * Let's do all this in Scheme?
// ! begin

fn car(x: &Exp) -> Option<Exp> {
    match x {
        Exp::List(l) => match l.get(0) {
            Some(y) => Some(y.clone()),
            None => None,
        },
        _ => None,
    }
}

fn cdr(x: &Exp) -> Option<Exp> {
    match x {
        Exp::List(l) => match l.get(1..) {
            Some(y) => Some(Exp::List(y.to_vec())),
            None => None,
        },
        _ => None,
    }
}

fn cons(x: &Exp, y: &Vec<Exp>) -> Option<Exp> {
    let mut to_append = y.clone();
    let mut res = vec![x.clone()];
    res.append(&mut to_append);
    Some(Exp::List(res))
}

// ! Oh, apply, unpacking...
*/

fn tokenize(str_exp: &str) -> VecDeque<String> {
    let res: VecDeque<_> = str_exp
        .replace("(", " ( ")
        .replace(")", " ) ")
        .split_whitespace()
        .map(String::from)
        .collect();
    res
}

fn atom(token: &str) -> Exp {
    match token.parse::<f64>() {
        Ok(num) => Exp::Number(num),
        Err(_) => Exp::Symbol(token.to_string()),
    }
}

fn gen_ast(tokens: &mut VecDeque<String>) -> Result<Exp, ScmErr> {
    if tokens.is_empty() {
        return Err(ScmErr::from("Unexpected EOF"));
    }

    let head = tokens.pop_front().unwrap();
    match head.as_ref() {
        // if we have a list ahead of us, we return that list
        "(" => {
            let mut res = VecDeque::new();
            loop {
                match tokens.get(0) {
                    Some(t) => match t.as_ref() {
                        ")" => break,
                        _ => match gen_ast(tokens) {
                            Ok(Exp::List(mut to_append)) => res.append(&mut to_append),
                            // recursion: deal with the tail of the list
                            // ! Attention: we are appending sub-expressions (including atoms) to the result
                            // Todo: refactor this function
                            Err(e) => return Err(e),
                            _ => return Err(ScmErr::from("Unknown Error")),
                        },
                    },
                    None => return Err(ScmErr::from("Mismatched parens")),
                }
            }
            tokens.pop_front(); // pop off ")"
            Ok(Exp::List(res))
        }
        ")" => Err(ScmErr::from("Extra ) found")),

        // if the head is a single atom, we return just the head
        _ => Ok(atom(&head)),
    }
}

fn parse(str_exp: &str) -> Result<Exp, ScmErr> {
    gen_ast(&mut tokenize(str_exp))
}

fn main() {
    // code goes here
    println!("Hello, rx_rs!");
}
