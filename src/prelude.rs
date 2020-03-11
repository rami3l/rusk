pub use crate::types::make_env_ptr;
use crate::types::*;
use std::process;

// * Primitive operators

// All primitive operators are fn(&[Exp]) -> Option<Exp>
// in order to fit into Exp::Primitive(fn(&[Exp]) -> Option<Exp>)

fn add(args: &[Exp]) -> Result<Exp, ScmErr> {
    let mut res = 0 as f64;
    for arg in args {
        if let &Exp::Number(x) = arg {
            res += x;
        } else {
            return Err(ScmErr::from("add: expected Exp::Number"));
        }
    }
    Ok(Exp::Number(res))
}

fn sub(pair: &[Exp]) -> Result<Exp, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Number(a - b)),
        _ => Err(ScmErr::from("sub: expected Exp::Number")),
    }
}

fn mul(args: &[Exp]) -> Result<Exp, ScmErr> {
    let mut res = 1 as f64;
    for arg in args {
        if let &Exp::Number(x) = arg {
            res *= x;
        } else {
            return Err(ScmErr::from("mul: expected Exp::Number"));
        }
    }
    Ok(Exp::Number(res))
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
        _ => Err(ScmErr::from("eq: expected Exp::Number")),
    }
}

fn lt(pair: &[Exp]) -> Result<Exp, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Bool(a < b)),
        _ => Err(ScmErr::from("lt: expected Exp::Number")),
    }
}

fn le(pair: &[Exp]) -> Result<Exp, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Bool(a <= b)),
        _ => Err(ScmErr::from("le: expected Exp::Number")),
    }
}

fn gt(pair: &[Exp]) -> Result<Exp, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Bool(a > b)),
        _ => Err(ScmErr::from("gt: expected Exp::Number")),
    }
}

fn ge(pair: &[Exp]) -> Result<Exp, ScmErr> {
    match pair {
        &[Exp::Number(a), Exp::Number(b)] => Ok(Exp::Bool(a >= b)),
        _ => Err(ScmErr::from("ge: expected Exp::Number")),
    }
}

fn car(args: &[Exp]) -> Result<Exp, ScmErr> {
    if args.len() != 1 {
        return Err(ScmErr::from("car: nothing to car"));
    }
    let pair = args.get(0).unwrap();
    match pair {
        Exp::List(list) => match list.get(0) {
            Some(res) => Ok(res.clone()),
            None => Err(ScmErr::from("car: expected a List of length 2")),
        },
        _ => Err(ScmErr::from("car: expected a List")),
    }
}

fn cdr(args: &[Exp]) -> Result<Exp, ScmErr> {
    if args.len() != 1 {
        return Err(ScmErr::from("cdr: nothing to cdr"));
    }
    let pair = args.get(0).unwrap();
    match pair {
        Exp::List(list) => match list.get(1) {
            Some(res) => Ok(res.clone()),
            None => Err(ScmErr::from("car: expected a List of length 2")),
        },
        _ => Err(ScmErr::from("cdr: expected a List")),
    }
}

fn cons(pair: &[Exp]) -> Result<Exp, ScmErr> {
    match pair {
        [a, b] => {
            let res: Vec<Exp> = [a.clone(), b.clone()].iter().cloned().collect();
            Ok(Exp::List(res))
        }
        _ => Err(ScmErr::from("cons: expected two Exp to cons")),
    }
}

fn is_null(args: &[Exp]) -> Result<Exp, ScmErr> {
    if args.is_empty() {
        return Err(ScmErr::from("null?: nothing to check"));
    }
    match args.get(0) {
        Some(Exp::List(list)) => match list.len() {
            0 => Ok(Exp::Bool(true)),
            _ => Ok(Exp::Bool(false)),
        },
        _ => Err(ScmErr::from("null?: expected a List")),
    }
}

fn display(args: &[Exp]) -> Result<Exp, ScmErr> {
    if args.len() != 1 {
        return Err(ScmErr::from("display: nothing to display"));
    }
    let res = args.first().unwrap();
    print!("{:?}", *res);
    Ok(Exp::Empty)
}

fn newline(args: &[Exp]) -> Result<Exp, ScmErr> {
    if !args.is_empty() {
        return Err(ScmErr::from("newline: too many arguments"));
    }
    println!();
    Ok(Exp::Empty)
}

fn exit(args: &[Exp]) -> Result<Exp, ScmErr> {
    let mut exit_code: i32 = 0;
    if !args.is_empty() {
        exit_code = match args[0] {
            Exp::Number(n) => n as i32,
            _ => return Err(ScmErr::from("exit: invalid exit code")),
        };
    }
    process::exit(exit_code);
}

// * Prelude

pub fn get_prelude() -> Env {
    let mut res = Env::from_outer(None);
    res.data = [
        ("+", Exp::Primitive(add)),
        ("-", Exp::Primitive(sub)),
        ("*", Exp::Primitive(mul)),
        ("/", Exp::Primitive(div)),
        ("=", Exp::Primitive(eq)),
        ("<", Exp::Primitive(lt)),
        ("<=", Exp::Primitive(le)),
        (">", Exp::Primitive(gt)),
        (">=", Exp::Primitive(ge)),
        ("car", Exp::Primitive(car)),
        ("cdr", Exp::Primitive(cdr)),
        ("cons", Exp::Primitive(cons)),
        ("null?", Exp::Primitive(is_null)),
        ("display", Exp::Primitive(display)),
        ("newline", Exp::Primitive(newline)),
        ("exit", Exp::Primitive(exit)),
        ("#t", Exp::Bool(true)),
        ("#f", Exp::Bool(false)),
        ("null", Exp::List(Vec::new())),
    ]
    .iter()
    .map(|(key, val)| (key.to_string(), val.clone()))
    .collect();

    res
}
