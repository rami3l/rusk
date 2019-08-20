use std::collections::{HashMap, VecDeque};
use std::iter::FromIterator;
use std::rc::Rc;
// use std::cell::RefCell;

#[macro_use]
extern crate lazy_static;

macro_rules! apply {
    // def apply(func, args): return func(args)
    ($func:expr, $args: expr) => {
        $func($args)
    };
}

// * Types

#[derive(Clone)]
enum Exp<'a> {
    // TODO: implement Display trait
    Bool(bool),
    Symbol(String),
    Number(f64),             // ! int unimplemented
    List(VecDeque<Exp<'a>>), // also used as AST
    Closure(ScmClosure<'a>),
    Primitive(fn(&[Exp<'a>]) -> Result<Exp<'a>, ScmErr>),
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

fn add<'a>(pair: &[Exp<'a>]) -> Result<Exp<'a>, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Number(a + b)),
        _ => Err(ScmErr::from("add: expected Exp::Number")),
    }
}

fn sub<'a>(pair: &[Exp<'a>]) -> Result<Exp<'a>, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Number(a - b)),
        _ => Err(ScmErr::from("sub: expected Exp::Number")),
    }
}

fn mul<'a>(pair: &[Exp<'a>]) -> Result<Exp<'a>, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Number(a * b)),
        _ => Err(ScmErr::from("mul: expected Exp::Number")),
    }
}

fn div<'a>(pair: &[Exp<'a>]) -> Result<Exp<'a>, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Number(a / b)),
        _ => Err(ScmErr::from("div: expected Exp::Number")),
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
        Exp::List(deque) => {
            let list: Vec<Exp> = deque.iter().map(|x| *x).collect();
            let Exp::Symbol(head) = &list[0];
            let tail = &list[1..];
            match head.as_ref() {
                "quote" => Ok(tail[0]),

                "lambda" => {
                    let closure_env = Env::new(Some(&env));
                    let closure = ScmClosure {
                        body: Rc::new(exp),
                        env: closure_env,
                    };
                    Ok(Exp::Closure(closure))
                }

                "define" => {
                    let Exp::Symbol(symbol) = tail[0]; // ! unpacking
                    let definition = tail[1];
                    env.data.insert(symbol, eval(definition, env).unwrap());
                    println!(">> Symbol defined");
                    // TODO: detailed info
                    Ok(Exp::Empty)
                }

                "if" => {
                    let [condition, then_, else_] = *tail;
                    // return eval(then_ if eval(condition, env) else else_, env)
                    match eval(condition, env) {
                        Ok(Exp::Bool(true)) => eval(then_, env),
                        Ok(Exp::Bool(false)) => eval(else_, env),
                        _ => Err(ScmErr::from("if: expected Exp::Bool")),
                    }
                }

                "cond" => {
                    for &item in tail {
                        let Exp::List(pair) = item;
                        let pair: Vec<_> = Vec::from_iter(pair.iter());
                        let [&condition, &then_] = &pair[..];
                        match eval(condition, env) {
                            Ok(Exp::Bool(true)) => return eval(then_, env),
                            Ok(Exp::Bool(false)) => continue,
                            Ok(Exp::Symbol(symbol)) => match symbol.as_ref() {
                                "else" => return eval(then_, env),
                                _ => continue,
                            },
                            _ => return Err(ScmErr::from("cond: expected Exp::Bool")),
                        }
                    }
                    Err(ScmErr::from("Missing else clause"))
                }

                "set!" => {
                    let [symbol, definition] = *tail;
                    let target = match env.find(&symbol) {
                        Some(&res) => &mut res,
                        None => &mut env,
                    };
                    let Exp::Symbol(key) = symbol;
                    target.data.insert(key, eval(definition, env).unwrap());
                    println!(">> Symbol set");
                    // TODO: detailed info
                    Ok(Exp::Empty)
                }

                _ => {
                    let func = eval(Exp::Symbol(*head), env).unwrap();
                    let args: Vec<Exp> = tail.iter().map(|i| eval(*i, env).unwrap()).collect();
                    apply_scm(func, &args[..])
                }
            }
        }
        _ => Err(ScmErr::from("eval: expected Exp")),
    }
}

fn apply_scm() {
    // TODO: implement this function
    // func can be Exp::Primitive or Exp::Closure
}

fn main() {
    // code goes here
    println!("Hello, rx_rs!");
}
