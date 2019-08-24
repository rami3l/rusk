use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::io;
// use std::iter::FromIterator;
// use std::rc::Rc;
// use std::cell::RefCell;

// #[macro_use]
// extern crate lazy_static;

macro_rules! apply {
    // def apply(func, args): return func(args)
    ($func:expr, $args: expr) => {
        $func($args)
    };
}

// * Types

#[derive(Clone)]
enum Exp {
    // TODO: implement Display trait
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
            Exp::Number(f) => format!("{}", f),
            Exp::List(l) => format!("{:?}", l),
            Exp::Closure(_) => "<Closure>".to_string(),
            Exp::Primitive(_) => "<Primitive>".to_string(),
            Exp::Empty => String::new(),
        };
        write!(f, "{}", res)
    }
}

#[derive(Clone)]
struct Env {
    data: HashMap<String, Exp>,
    outer: Option<Box<Env>>,
}

impl Env {
    fn new(outer: Option<Box<Env>>) -> Env {
        Env {
            data: HashMap::new(),
            outer,
        }
    }
    fn find(&self, symbol: &Exp) -> Option<Box<Env>> {
        // find the innermost Env where symbol appears
        match symbol {
            Exp::Symbol(s) => match self.data.get(s) {
                Some(_) => Some(Box::new(self.clone())),
                None => match &self.outer {
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

#[derive(Clone)]
struct ScmClosure {
    body: Box<Exp>,
    env: Env,
}

enum ScmErr {
    Reason(String),
}

impl ScmErr {
    fn from(reason: &str) -> ScmErr {
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

fn gen_ast(tokens: &VecDeque<String>) -> Result<Exp, ScmErr> {
    if tokens.is_empty() {
        return Err(ScmErr::from("Unexpected EOF"));
    }

    let mut tokens = tokens.clone();
    let head = tokens.pop_front().unwrap();
    match head.as_ref() {
        // if we have a list ahead of us, we return that list
        "(" => {
            let mut res = VecDeque::new();
            loop {
                match tokens.get(0) {
                    Some(t) => match t.as_ref() {
                        ")" => break,
                        _ => match gen_ast(&tokens) {
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
    let ast = tokenize(str_exp);
    gen_ast(&ast)
}

// * Primitive operators

// All primitive operators are fn(&[Exp]) -> Option<Exp>
// in order to fit into Exp::Primitive(fn(&[Exp]) -> Option<Exp>)

fn add(pair: &[Exp]) -> Result<Exp, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Number(a + b)),
        _ => Err(ScmErr::from("add: expected Exp::Number")),
    }
}

fn sub(pair: &[Exp]) -> Result<Exp, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Number(a - b)),
        _ => Err(ScmErr::from("sub: expected Exp::Number")),
    }
}

fn mul(pair: &[Exp]) -> Result<Exp, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Number(a * b)),
        _ => Err(ScmErr::from("mul: expected Exp::Number")),
    }
}

fn div(pair: &[Exp]) -> Result<Exp, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Number(a / b)),
        _ => Err(ScmErr::from("div: expected Exp::Number")),
    }
}

// * Prelude

fn get_prelude() -> Env {
    let mut res = Env::new(None);
    let data = &mut res.data;

    // initializing the environment
    data.insert(String::from("+"), Exp::Primitive(add));
    data.insert(String::from("-"), Exp::Primitive(sub));
    data.insert(String::from("*"), Exp::Primitive(mul));
    data.insert(String::from("/"), Exp::Primitive(div));

    res
}

fn eval(exp: Exp, env: &mut Env) -> Result<Exp, ScmErr> {
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
            let list: Vec<Exp> = deque.iter().map(|x| x.clone()).collect();
            let head = match list.get(0) {
                Some(Exp::Symbol(res)) => res,
                _ => return Err(ScmErr::from("eval: expected a non-empty list of Symbol")),
            };
            let tail = &list[1..];
            match head.as_ref() {
                "quote" => match tail.get(0) {
                    Some(res) => Ok(res.clone()),
                    None => Err(ScmErr::from("quote: nothing to quote")),
                },

                "lambda" => {
                    let tail_deque: VecDeque<Exp> = tail.iter().map(|x| x.clone()).collect();
                    let tail = Exp::List(tail_deque);
                    let closure = ScmClosure {
                        body: Box::new(tail),
                        env: Env::new(Some(Box::new(env.clone()))),
                    };
                    Ok(Exp::Closure(closure))
                }

                "define" => {
                    let symbol = match tail.get(0) {
                        Some(Exp::Symbol(res)) => res.clone(),
                        None => return Err(ScmErr::from("define: nothing to define")),
                        _ => return Err(ScmErr::from("define: expected Symbol")),
                    };
                    let definition = match tail.get(1) {
                        Some(res) => res.clone(),
                        None => return Err(ScmErr::from("define: nothing to define")),
                    };
                    let eval_definition = eval(definition, env).unwrap();
                    env.data.insert(symbol, eval_definition);
                    println!(">> Symbol defined");
                    // TODO: detailed info
                    Ok(Exp::Empty)
                }

                "if" => {
                    let condition = match tail.get(0) {
                        Some(res) => res.clone(),
                        None => return Err(ScmErr::from("if: missing condition")),
                    };
                    let then_ = match tail.get(1) {
                        Some(res) => res.clone(),
                        None => return Err(ScmErr::from("if: missing then clause")),
                    };
                    let else_ = match tail.get(2) {
                        Some(res) => res.clone(),
                        None => return Err(ScmErr::from("if: missing else clause")),
                    };
                    // return eval(then_ if eval(condition, env) else else_, env)
                    match eval(condition, env) {
                        Ok(Exp::Bool(true)) => eval(then_, env),
                        Ok(Exp::Bool(false)) => eval(else_, env),
                        _ => Err(ScmErr::from("if: expected Exp::Bool")),
                    }
                }

                "cond" => {
                    for item in tail.iter() {
                        let pair = match item {
                            Exp::List(res) => res.clone(),
                            _ => return Err(ScmErr::from("cond: expected pairs")),
                        };
                        let condition = match pair.get(0) {
                            Some(res) => res.clone(),
                            None => return Err(ScmErr::from("cond: missing condition")),
                        };
                        let then_ = match pair.get(1) {
                            Some(res) => res.clone(),
                            None => return Err(ScmErr::from("cond: missing then clause")),
                        };
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
                    let symbol = match tail.get(0) {
                        Some(res) => res.clone(),
                        None => return Err(ScmErr::from("define: nothing to define")),
                    };
                    let definition = match tail.get(1) {
                        Some(res) => res.clone(),
                        None => return Err(ScmErr::from("define: nothing to define")),
                    };
                    let eval_definition = eval(definition, env).unwrap();
                    let target = match env.find(&symbol) {
                        // ! Box::leak()
                        Some(res) => Box::leak(res),
                        None => env,
                    };
                    let key = match symbol {
                        Exp::Symbol(res) => res,
                        _ => return Err(ScmErr::from("define: expected Symbol")),
                    };
                    target.data.insert(key, eval_definition);
                    println!(">> Symbol set");
                    // TODO: detailed info
                    Ok(Exp::Empty)
                }

                _ => {
                    let func = eval(Exp::Symbol(head.clone()), env).unwrap();
                    let args: Vec<Exp> =
                        tail.iter().map(|i| eval(i.clone(), env).unwrap()).collect();
                    apply_scm(func, &args[..])
                }
            }
        }
        _ => Err(ScmErr::from("eval: expected Exp")),
    }
}

fn apply_scm(func: Exp, args: &[Exp]) -> Result<Exp, ScmErr> {
    // TODO: implement this function
    // func can be Exp::Primitive or Exp::Closure
    match func {
        Exp::Primitive(prim) => apply!(prim, args),

        Exp::Closure(clos) => match *clos.body {
            Exp::List(body) => match body.get(0) {
                Some(Exp::List(vars)) => {
                    let mut local_env = clos.env.clone();
                    for (var, arg) in vars.iter().zip(args) {
                        let var = var.clone();
                        let arg = arg.clone();
                        match var {
                            Exp::Symbol(i) => local_env.data.insert(i, arg),
                            _ => {
                                return Err(ScmErr::from(
                                    "closure unpacking error: expected a list of Symbol",
                                ))
                            }
                        };
                    }
                    match body.get(1) {
                        Some(exp) => eval(exp.clone(), &mut local_env),
                        None => {
                            return Err(ScmErr::from("closure unpacking error: missing definition"))
                        }
                    }
                }
                _ => Err(ScmErr::from(
                    "closure unpacking error: expected a non-empty list",
                )),
            },
            _ => Err(ScmErr::from("closure unpacking error: expected a list")),
        },
        _ => Err(ScmErr::from(
            "apply_scm: a function can only be Exp::Primitive or Exp::Closure",
        )),
    }
}

fn repl() {
    let mut count = 0;
    let mut global_env: Env = get_prelude();
    println!("<rx.rs>");
    loop {
        count += 1;
        print!("#;{}> ", count);
        // ! read input
        let mut str_exp = String::new();
        match io::stdin().read_line(&mut str_exp) {
            Ok(_) => (),
            Err(e) => println!("Input: {}", e),
        }
        let str_exp = str_exp.trim();
        match str_exp {
            ",q" => {
                println!("Quitting...");
                break;
            }
            _ => match parse(str_exp) {
                Ok(exp) => {
                    let val = eval(exp, &mut global_env);
                    println!("=> {:?}", val);
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                    continue;
                }
            },
        };
    }
}

fn main() {
    // code goes here
    // println!("Hello, rx_rs!");
    repl();
}
