use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
// use std::cell::RefCell;

#[macro_use]
extern crate lazy_static;

macro_rules! apply {
    // def apply(func, args): return func(args)
    ($func:expr, $args: expr)  => {
        $func($args)
    };
}

// * Types

#[derive(Clone)]
enum Exp<'a> {
    Bool(bool),
    Symbol(String),
    Number(f64),         // ! int unimplemented
    List(VecDeque<Exp<'a>>), // also used as AST
    Closure(ScmClosure<'a>),
    Primitive(fn(&[Exp<'a>]) -> Option<Exp<'a>>),
    Empty,
}

#[derive(Clone)]
struct Env<'a> {
    data: HashMap<String, Exp<'a>>,
    outer: Option<&'a Env<'a>>,
}

impl<'a> Env<'a> {
    fn new(outer: Option<&'a Env<'a>>) -> Env {
        Env {
            data: HashMap::new(),
            outer,
        }
    }
    fn find(&self, symbol: &Exp) -> Option<&Env<'a>> {
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

    fn lookup(&self, symbol: &Exp) -> Option<Exp<'a>> {
        match symbol {
            Exp::Symbol(s) => match self.data.get(s) {
                Some(exp) => Some(exp.clone()),
                None => None,
            },
            _ => None,
        }
    }
}

#[derive(Clone)]
struct ScmClosure<'a> {
    body: Rc<Exp<'a>>,
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

// * Parsing

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


// * Primitive operators

// All primitive operators are fn(&[Exp]) -> Option<Exp>
// in order to fit into Exp::Primitive(fn(&[Exp]) -> Option<Exp>)

fn add<'a>(pair: &[Exp<'a>]) -> Option<Exp<'a>> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Some(Exp::Number(a+b)),
        _ => None,
    }
}

fn sub<'a>(pair: &[Exp<'a>]) -> Option<Exp<'a>> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Some(Exp::Number(a-b)),
        _ => None,
    }
}

fn mul<'a>(pair: &[Exp<'a>]) -> Option<Exp<'a>> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Some(Exp::Number(a*b)),
        _ => None,
    }
}

fn div<'a>(pair: &[Exp<'a>]) -> Option<Exp<'a>> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Some(Exp::Number(a-b)),
        _ => None,
    }
}

// * Prelude

fn get_prelude() -> Env<'static> {
    let mut res = Env::new(None);
    let data = &mut res.data;

    // initializing the environment
    data.insert(String::from("+"), Exp::Primitive(add));
    data.insert(String::from("-"), Exp::Primitive(sub));
    data.insert(String::from("*"), Exp::Primitive(mul));
    data.insert(String::from("/"), Exp::Primitive(div));

    res
}

lazy_static! {
    static ref GLOBAL_ENV: Env<'static> = get_prelude();
}


fn eval<'a>(exp: Exp<'a>, env: Env<'a>) -> Result<Exp<'a>, ScmErr> {
    match exp {
        Exp::Number(_) => Ok(exp),
        Exp::Symbol(_) => {
            match env.lookup(&exp) {
                Some(res) => Ok(res),
                None => Err(ScmErr::from("Symbol undefined")),
                // TODO: detailed info
            }
        }
        Exp::List(list) => {
            if let Exp::Symbol(head) = &list[0] {
            match head.as_ref() {
                "quote" => Ok(list[1]),

                "lambda" => {
                    let closure_env = Env::new(Some(&env));
                    let closure = ScmClosure {
                        body: Rc::new(exp),
                        env: closure_env,
                    };
                    Ok(Exp::Closure(closure))
                }

                "define" => {
                    let Exp::Symbol(symbol) = &list[1]; // ! unpacking
                    let definition = &list[2];
                    env.data.insert(*symbol, eval(*definition, env).unwrap());
                    println!("Symbol defined");
                    // TODO: detailed info
                    Ok(Exp::Empty)
                }

                "if" => {
                    let condition = &list[1];
                    let then_ = &list[2];
                    let else_ = &list[3];
                    // return eval(then_ if eval(condition, env) else else_, env)
                }

                _ => Err(ScmErr::from("Unknown keyword")),
            }}
            else {
                Err(ScmErr::from("Expected a Symbol head for a List"))
            }
        }
    }
}

fn main() {
    // code goes here
    println!("Hello, rx_rs!");
}
