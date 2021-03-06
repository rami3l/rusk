use crate::prelude::make_env_ptr;
use crate::types::*;
use std::rc::Rc;

pub fn eval(exp: Exp, env: RcRefCell<Env>) -> Result<Exp, ScmErr> {
    match exp {
        n @ Exp::Number(_) => Ok(n),

        ref s @ Exp::Symbol(_) => env
            .borrow()
            .lookup(s)
            .ok_or(ScmErr::from(&format!("eval: Symbol {} undefined", s))),

        Exp::List(list) => {
            if list.is_empty() {
                return Err(ScmErr::from("eval: expect a non-empty list"));
            }

            let tail = &list[1..];

            // A tiny closure to send a lambda expression to apply
            let handle_lambda = |exp: Exp| {
                let func = eval(exp, Rc::clone(&env))?;
                let args: Vec<Exp> = tail
                    .iter()
                    .map(|i| eval(i.clone(), Rc::clone(&env)).unwrap())
                    .collect();
                apply(func, &args[..])
            };

            let head = match list.first() {
                Some(Exp::Symbol(res)) => res,
                Some(lambda @ Exp::List(_)) => {
                    // head is an inline lambda expression
                    return handle_lambda(lambda.clone());
                }
                _ => return Err(ScmErr::from("eval: head of the list is not a function")),
            };

            match head.as_ref() {
                // ! This is a WRONG quote.
                // TODO: implement proper cons structure.
                "quote" => tail
                    .first()
                    .ok_or_else(|| ScmErr::from("quote: nothing to quote"))
                    .cloned(),

                "lambda" => {
                    let tail: Vec<Exp> = tail.iter().cloned().collect();
                    let closure = ScmClosure {
                        body: Box::new(Exp::List(tail)),
                        env: Env::from_outer(Some(Rc::clone(&env))),
                        // Here we want to clone a pointer, not to clone an Env.
                    };
                    Ok(Exp::Closure(closure))
                }

                "define" => {
                    match tail {
                        [symbol, definition] => {
                            let symbol_str = match symbol {
                                Exp::Symbol(res) => res.clone(),
                                _ => return Err(ScmErr::from("define: expected Symbol")),
                            };
                            let eval_definition = eval(definition.clone(), Rc::clone(&env))?;
                            env.borrow_mut().data.insert(symbol_str, eval_definition);
                        }
                        _ => return Err(ScmErr::from("define: nothing to define")),
                    };
                    Ok(Exp::Empty)
                }

                "set!" => {
                    let symbol = tail
                        .get(0)
                        .ok_or(ScmErr::from("set!: nothing to set!"))?
                        .clone();
                    let definition = tail
                        .get(1)
                        .ok_or(ScmErr::from("set!: nothing to set!"))?
                        .clone();
                    let eval_definition = eval(definition, Rc::clone(&env))?;
                    let key = match symbol.clone() {
                        Exp::Symbol(res) => res,
                        _ => return Err(ScmErr::from("set!: expected Symbol")),
                    };

                    // Find the innermost Env in which a symbol is defined starting from the current Env.
                    let target: RcRefCell<Env> = {
                        let mut current = Rc::clone(&env);
                        loop {
                            let outer = match &current.borrow().outer {
                                Some(x) => Rc::clone(&x),
                                None => break Rc::clone(&current),
                            };
                            match current.borrow().data.get(&key) {
                                Some(_) => break Rc::clone(&current),
                                None => (),
                            };
                            current = outer;
                        }
                    };
                    target.borrow_mut().data.insert(key, eval_definition);
                    /*
                    // * print details
                    println!(
                        ">> Symbol \"{:?}\" set to {:?}",
                        symbol,
                        env.borrow().lookup(&symbol).unwrap()
                    );
                    */
                    Ok(Exp::Empty)
                }

                "if" => {
                    let condition = tail
                        .get(0)
                        .ok_or(ScmErr::from("if: missing condition"))?
                        .clone();
                    let then_ = tail
                        .get(1)
                        .ok_or(ScmErr::from("if: missing then clause"))?
                        .clone();
                    let else_ = tail
                        .get(2)
                        .ok_or(ScmErr::from("if: missing else clause"))?
                        .clone();
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
                        let then_ = pair
                            .get(1)
                            .ok_or(ScmErr::from("cond: missing then clause"))?
                            .clone();
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
                    Err(ScmErr::from("cond: missing else clause"))
                }

                "begin" => tail.iter().fold(Ok(Exp::Empty), |_seed, item| {
                    eval(item.clone(), Rc::clone(&env))
                }),

                _ => {
                    // head is a closure
                    handle_lambda(Exp::Symbol(head.clone()))
                }
            }
        }
        _ => Err(ScmErr::from("eval: unexpected Exp")),
    }
}

fn apply(func: Exp, args: &[Exp]) -> Result<Exp, ScmErr> {
    // func can be Exp::Primitive or Exp::Closure
    match func {
        Exp::Primitive(prim) => prim(args),

        Exp::Closure(clos) => match *clos.body {
            Exp::List(body) => match body.first() {
                Some(Exp::List(vars)) => {
                    let local_env = make_env_ptr(clos.env.clone());
                    for (var, arg) in vars.iter().zip(args) {
                        match var {
                            Exp::Symbol(i) => {
                                local_env.borrow_mut().data.insert(i.clone(), arg.clone())
                            }
                            _ => {
                                return Err(ScmErr::from(
                                    "closure unpacking error: expected a list of Symbol's",
                                ))
                            }
                        };
                    }
                    let definition = &body[1..];
                    if definition.is_empty() {
                        return Err(ScmErr::from("closure unpacking error: missing definition"));
                    }
                    definition.iter().fold(Ok(Exp::Empty), |_seed, exp| {
                        eval(exp.clone(), Rc::clone(&local_env))
                    })
                }
                _ => Err(ScmErr::from(
                    "closure unpacking error: expected a non-empty list",
                )),
            },
            _ => Err(ScmErr::from("closure unpacking error: expected a list")),
        },
        _ => Err(ScmErr::from(
            "apply: a function can only be Exp::Primitive or Exp::Closure",
        )),
    }
}
