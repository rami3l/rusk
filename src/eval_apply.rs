use crate::types::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

pub fn eval(exp: Exp, env: RcRefCellBox<Env>) -> Result<Exp, ScmErr> {
    match exp {
        Exp::Number(_) => Ok(exp),
        Exp::Symbol(s) => match env.borrow().lookup(&Exp::Symbol(s.clone())) {
            Some(res) => Ok(res),
            None => Err(ScmErr::from(&format!("eval: Symbol \"{}\" undefined", s))),
        },
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
                        env: Env::from_outer(Some(Rc::clone(&env))),
                        // ! Here we want to clone a pointer, not to clone an Env.
                    };
                    Ok(Exp::Closure(closure))
                }

                "define" => {
                    let symbol = match tail.get(0) {
                        Some(res) => res.clone(),
                        None => return Err(ScmErr::from("define: nothing to define")),
                    };
                    let symbol_str = match symbol.clone() {
                        Exp::Symbol(res) => res.clone(),
                        _ => return Err(ScmErr::from("define: expected Symbol")),
                    };
                    let definition = match tail.get(1) {
                        Some(res) => res.clone(),
                        None => return Err(ScmErr::from("define: nothing to define")),
                    };
                    let eval_definition = match eval(definition, Rc::clone(&env)) {
                        Ok(res) => res,
                        Err(e) => return Err(e),
                    };
                    env.borrow_mut()
                        .data
                        .insert(symbol_str.clone(), eval_definition);
                    println!(
                        ">> Symbol \"{:?}\" defined as {:?}",
                        symbol,
                        env.borrow().lookup(&symbol).unwrap()
                    );
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
                    match eval(condition, Rc::clone(&env)) {
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
                        match eval(condition, Rc::clone(&env)) {
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
                    let eval_definition = match eval(definition, Rc::clone(&env)) {
                        Ok(res) => res,
                        Err(e) => return Err(e),
                    };
                    let key = match symbol.clone() {
                        Exp::Symbol(res) => res,
                        _ => return Err(ScmErr::from("set!: expected Symbol")),
                    };
                    let target: RcRefCellBox<Env> = {
                        let mut current = Rc::clone(&env);
                        let res;
                        loop {
                            let outer = match &current.borrow().outer {
                                Some(x) => Rc::clone(&x),
                                None => {
                                    res = Rc::clone(&current);
                                    break;
                                }
                            };
                            match current.borrow().data.get(&key) {
                                Some(_) => {
                                    res = Rc::clone(&current);
                                    break;
                                }
                                None => (),
                            };
                            current = outer;
                        }
                        res
                    };
                    target.borrow_mut().data.insert(key, eval_definition);
                    println!(
                        ">> Symbol \"{:?}\" set to {:?}",
                        symbol,
                        env.borrow().lookup(&symbol).unwrap()
                    );
                    Ok(Exp::Empty)
                }

                _ => {
                    let func = match eval(Exp::Symbol(head.clone()), Rc::clone(&env)) {
                        Ok(res) => res,
                        Err(e) => return Err(e),
                    };
                    let args: Vec<Exp> = tail
                        .iter()
                        .map(|i| eval(i.clone(), Rc::clone(&env)).unwrap())
                        .collect();
                    apply(func, &args[..])
                }
            }
        }
        _ => Err(ScmErr::from("eval: expected Exp")),
    }
}

fn apply(func: Exp, args: &[Exp]) -> Result<Exp, ScmErr> {
    // func can be Exp::Primitive or Exp::Closure
    match func {
        Exp::Primitive(prim) => prim(args),

        Exp::Closure(clos) => match *clos.body {
            Exp::List(body) => match body.get(0) {
                Some(Exp::List(vars)) => {
                    let local_env = Rc::new(RefCell::new(Box::new(clos.env.clone())));
                    for (var, arg) in vars.iter().zip(args) {
                        let var = var.clone();
                        let arg = arg.clone();
                        match var {
                            Exp::Symbol(i) => local_env.borrow_mut().data.insert(i, arg),
                            _ => {
                                return Err(ScmErr::from(
                                    "closure unpacking error: expected a list of Symbol",
                                ))
                            }
                        };
                    }
                    match body.get(1) {
                        Some(exp) => eval(exp.clone(), local_env),
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
