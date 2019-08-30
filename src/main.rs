use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::io::{self, Write};
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
        // find the definition of a symbol
        match symbol {
            Exp::Symbol(s) => match self.data.get(s) {
                Some(def) => Some(def.clone()),
                None => match &self.outer {
                    Some(outer) => outer.lookup(symbol),
                    None => None,
                },
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
        return Err(ScmErr::from("gen_ast: Unexpected EOF"));
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
                            Ok(Exp::List(l)) => res.push_back(Exp::List(l)),
                            Ok(Exp::Symbol(s)) => res.push_back(Exp::Symbol(s)),
                            Ok(Exp::Number(f)) => res.push_back(Exp::Number(f)),
                            // recursion: deal with the tail of the list
                            // ! Attention: we are appending sub-expressions (including atoms) to the result
                            // Todo: refactor this function
                            Err(e) => return Err(e),
                            _ => {
                                return Err(ScmErr::from(
                                    "gen_ast: Expected a deque of Symbol, Number or List",
                                ))
                            }
                        },
                    },
                    None => return Err(ScmErr::from("gen_ast: Mismatched parens")),
                }
            }
            tokens.pop_front(); // pop off ")"
            Ok(Exp::List(res))
        }
        ")" => Err(ScmErr::from("gen_ast: Extra \")\" found")),

        // if the head is a single atom, we return just the head
        _ => Ok(atom(&head)),
    }
}

fn parse(str_exp: &str) -> Result<Exp, ScmErr> {
    let mut ast = tokenize(str_exp);
    gen_ast(&mut ast)
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

fn eq(pair: &[Exp]) -> Result<Exp, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Bool(a == b)),
        _ => Err(ScmErr::from("eq: expected Exp::Bool")),
    }
}

fn lt(pair: &[Exp]) -> Result<Exp, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Bool(a < b)),
        _ => Err(ScmErr::from("lt: expected Exp::Bool")),
    }
}

fn le(pair: &[Exp]) -> Result<Exp, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Bool(a <= b)),
        _ => Err(ScmErr::from("le: expected Exp::Bool")),
    }
}

fn gt(pair: &[Exp]) -> Result<Exp, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Bool(a > b)),
        _ => Err(ScmErr::from("gt: expected Exp::Bool")),
    }
}

fn ge(pair: &[Exp]) -> Result<Exp, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Bool(a >= b)),
        _ => Err(ScmErr::from("ge: expected Exp::Bool")),
    }
}

// ! begin

fn car(args: &[Exp]) -> Result<Exp, ScmErr> {
    if args.is_empty() {
        return Err(ScmErr::from("car: nothing to car"));
    }
    let pair = args.get(0).unwrap();
    match pair {
        Exp::List(deque) => match deque.get(0) {
            Some(res) => Ok(res.clone()),
            None => Err(ScmErr::from("car: expected a List of length 2")),
        },
        _ => Err(ScmErr::from("car: expected a List")),
    }
}

fn cdr(args: &[Exp]) -> Result<Exp, ScmErr> {
    if args.is_empty() {
        return Err(ScmErr::from("cdr: nothing to cdr"));
    }
    let pair = args.get(0).unwrap();
    match pair {
        Exp::List(deque) => match deque.get(1) {
            Some(res) => Ok(res.clone()),
            None => Err(ScmErr::from("car: expected a List of length 2")),
        },
        _ => Err(ScmErr::from("cdr: expected a List")),
    }
}

fn cons(pair: &[Exp]) -> Result<Exp, ScmErr> {
    match pair {
        [a, b] => {
            let res: VecDeque<Exp> = [a.clone(), b.clone()].iter().map(|x| x.clone()).collect();
            Ok(Exp::List(res))
        }
        _ => Err(ScmErr::from("cons: expected two Exp to cons")),
    }
}

fn is_null(args: &[Exp]) -> Result<Exp, ScmErr> {
    if args.is_empty() {
        return Err(ScmErr::from("empty?: nothing to check"));
    }
    match args.get(0) {
        Some(Exp::List(list)) => match list.len() {
            0 => Ok(Exp::Bool(true)),
            _ => Ok(Exp::Bool(false)),
        },
        _ => Err(ScmErr::from("empty?: expected a List")),
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
    data.insert(String::from("="), Exp::Primitive(eq));
    data.insert(String::from("<"), Exp::Primitive(lt));
    data.insert(String::from("<="), Exp::Primitive(le));
    data.insert(String::from(">"), Exp::Primitive(gt));
    data.insert(String::from(">="), Exp::Primitive(ge));

    data.insert(String::from("car"), Exp::Primitive(car));
    data.insert(String::from("cdr"), Exp::Primitive(cdr));
    data.insert(String::from("cons"), Exp::Primitive(cons));

    data.insert(String::from("null?"), Exp::Primitive(is_null));

    data.insert(String::from("#t"), Exp::Bool(true));
    data.insert(String::from("#f"), Exp::Bool(false));

    data.insert(String::from("null"), Exp::List(VecDeque::new()));

    res
}

fn eval(exp: Exp, env: &mut Env) -> Result<Exp, ScmErr> {
    match exp {
        Exp::Number(_) => Ok(exp),
        Exp::Symbol(s) => {
            match env.lookup(&Exp::Symbol(s.clone())) {
                Some(res) => Ok(res),
                None => Err(ScmErr::from(&format!("eval: Symbol \"{}\" undefined", s))),
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
                        // ! To fix: we want to clone a pointer, from &mut Env to Box<Env>, not to clone an Env.
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
                    let eval_definition = match eval(definition, env) {
                        Ok(res) => res,
                        Err(e) => return Err(e),
                    };
                    env.data.insert(symbol.clone(), eval_definition);
                    println!(">> Symbol \"{}\" defined", symbol);
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
                        let then_ = match pair.get(1) {
                            Some(res) => res.clone(),
                            None => return Err(ScmErr::from("cond: missing then clause")),
                        };
                        let condition = match pair.get(0) {
                            Some(Exp::Symbol(s)) => match s.as_ref() {
                                "else" => return eval(then_, env),
                                _ => Exp::Symbol(s.clone()),
                            },
                            Some(res) => res.clone(),
                            None => return Err(ScmErr::from("cond: missing condition")),
                        };
                        match eval(condition, env) {
                            Ok(Exp::Bool(true)) => return eval(then_, env),
                            Ok(Exp::Bool(false)) => continue,
                            _ => return Err(ScmErr::from("cond: expected Exp::Bool")),
                        }
                    }
                    Err(ScmErr::from("Missing else clause"))
                }

                "set!" => {
                    let symbol = match tail.get(0) {
                        Some(res) => res.clone(),
                        None => return Err(ScmErr::from("set!: nothing to set!")),
                    };
                    let definition = match tail.get(1) {
                        Some(res) => res.clone(),
                        None => return Err(ScmErr::from("set!: nothing to set!")),
                    };
                    let eval_definition = match eval(definition, env) {
                        Ok(res) => res,
                        Err(e) => return Err(e),
                    };
                    let target = match env.find(&symbol) {
                        // ! Box::leak()
                        Some(res) => Box::leak(res),
                        None => env,
                    };
                    let key = match symbol {
                        Exp::Symbol(res) => res,
                        _ => return Err(ScmErr::from("set!: expected Symbol")),
                    };
                    target.data.insert(key, eval_definition);
                    println!(">> Symbol set");
                    // TODO: detailed info
                    Ok(Exp::Empty)
                }

                _ => {
                    let func = match eval(Exp::Symbol(head.clone()), env) {
                        Ok(res) => res,
                        Err(e) => return Err(e),
                    };
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
        io::stdout().flush().unwrap();
        // ! read input
        let mut str_exp = String::new();
        io::stdin().read_line(&mut str_exp).unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let left = "(+ 1 2)";
        let right: VecDeque<String> = vec!["(", "+", "1", "2", ")"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        assert_eq!(tokenize(left), right)
    }

    #[test]
    fn test_parse() {
        let left = "(+ 1 2)";
        let right: VecDeque<Exp> = vec![
            Exp::Symbol("+".to_string()),
            Exp::Number(1 as f64),
            Exp::Number(2 as f64),
        ]
        .iter()
        .map(|x| x.clone())
        .collect();
        let left = match parse(left) {
            Ok(Exp::List(l)) => l,
            _ => panic!("should parse to a list"),
        };
        for (l, r) in left.iter().zip(right.iter()) {
            match (l, r) {
                (Exp::Symbol(ls), Exp::Symbol(rs)) => assert_eq!(ls, rs),
                (Exp::Number(lf), Exp::Number(rf)) => assert_eq!(lf, rf),
                _ => panic!("should be Symbol or Number"),
            }
        }
    }

    fn check_io_str(input: &str, output: &str, env: &mut Env) {
        let str_exp = input.to_string();
        let right = output.to_string();
        let left = match parse(&str_exp) {
            Ok(exp) => {
                let val = eval(exp, env);
                format!("{:?}", val)
            }
            Err(e) => format!("Error: {:?}", e),
        };
        assert_eq!(left, right);
    }

    #[test]
    fn test_plus() {
        let mut env: Env = get_prelude();
        let env = &mut env;
        check_io_str("(+ 1 2)", "Ok(3)", env);
    }

    #[test]
    fn test_plus_nested() {
        let mut env: Env = get_prelude();
        let env = &mut env;
        check_io_str("(+ 1 (* 2 3))", "Ok(7)", env);
    }

    #[test]
    fn test_define_val() {
        let mut env: Env = get_prelude();
        let env = &mut env;
        check_io_str("(define x 3)", "Ok()", env);
        check_io_str("x", "Ok(3)", env);
        check_io_str("(+ x 1)", "Ok(4)", env);
    }

    #[test]
    fn test_define_proc_basic() {
        let mut env: Env = get_prelude();
        let env = &mut env;
        check_io_str("(define x 3)", "Ok()", env);
        check_io_str("x", "Ok(3)", env);
        check_io_str("(define one (lambda () 1))", "Ok()", env);
        check_io_str("(one)", "Ok(1)", env);
        check_io_str("(+ (one) (+ 2 x))", "Ok(6)", env);
    }

    #[test]
    fn test_define_proc_call_prim() {
        let mut env: Env = get_prelude();
        let env = &mut env;
        check_io_str("(define x 3)", "Ok()", env);
        check_io_str("x", "Ok(3)", env);
        check_io_str("(define inc (lambda (x) (+ x 1)))", "Ok()", env);
        check_io_str("(inc 100)", "Ok(101)", env);
        check_io_str("(inc x)", "Ok(4)", env);
    }

    #[test]
    fn test_cond() {
        let mut env: Env = get_prelude();
        let env = &mut env;
        check_io_str("(if #t 123 wtf)", "Ok(123)", env);
        check_io_str("(if #f wtf 123)", "Ok(123)", env);
        check_io_str(
            "(cond (#f wtf0) (#f wtf1) (#t 456) (else wtf3))",
            "Ok(456)",
            env,
        );
        check_io_str(
            "(cond (#f wtf0) (#f wtf1) (#f wtf2) (else 789))",
            "Ok(789)",
            env,
        );
    }

    #[test]
    fn test_eq() {
        let mut env: Env = get_prelude();
        let env = &mut env;
        check_io_str("(define one (lambda () 1))", "Ok()", env);
        check_io_str("(= 1 1)", "Ok(true)", env);
        check_io_str("(= 1 (one))", "Ok(true)", env);
        check_io_str("(if (= 1 (one)) 123 wtf)", "Ok(123)", env);
        check_io_str("(if (= (one) (+ 4 5)) wtf 123)", "Ok(123)", env);
    }

    #[test]
    fn test_cons() {
        let mut env: Env = get_prelude();
        let env = &mut env;
        check_io_str("(car (cons 123 456))", "Ok(123)", env);
        check_io_str("(cdr (cons 123 456))", "Ok(456)", env);
        check_io_str("(define p (cons (cons 1 2) (cons 3 4)))", "Ok()", env);
        check_io_str("(cdr (car p))", "Ok(2)", env);
        check_io_str("(cdr p)", "Ok([3, 4])", env);
        check_io_str("p", "Ok([[1, 2], [3, 4]])", env);
        check_io_str(
            "(define l (cons 1 (cons 2 (cons 3 null))))",
            "Ok()",
            env,
        );
        check_io_str("(car (cdr l))", "Ok(2)", env);
        check_io_str("(cdr (cdr (cdr l)))", "Ok([])", env);
    }

    #[test]
    fn test_fibonacci() {
        let mut env: Env = get_prelude();
        let env = &mut env;
        check_io_str(
            "(define fib (lambda (n) (if (< n 2) 1 (+ (fib (- n 1)) (fib (- n 2))))))",
            "Ok()",
            env,
        );
        check_io_str("(fib 20)", "Ok(10946)", env);
        check_io_str(
            "(define range (lambda (a b) (if (= a b) (quote ()) (cons a (range (+ a 1) b)))))",
            "Ok()",
            env,
        );
        check_io_str(
            "(define map (lambda (f l) (if (null? l) null (cons (f (car l)) (map f (cdr l))))))",
            "Ok()",
            env,
        );
        check_io_str(
            "(range 0 10)",
            "Ok([0, 1, 2, 3, 4, 5, 6, 7, 8, 9])",
            env,
        );
        // check_io_str("(map fib (range 0 20))", "Ok([3, 4])", env);
    }
}
