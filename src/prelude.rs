use crate::types::*;
use std::collections::VecDeque;
use std::process;

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

    data.insert(String::from("exit"), Exp::Primitive(exit));

    data.insert(String::from("#t"), Exp::Bool(true));
    data.insert(String::from("#f"), Exp::Bool(false));

    data.insert(String::from("null"), Exp::List(VecDeque::new()));

    res
}
