use crate::eval_apply::eval;
use crate::parser::InPort;
use crate::types::{Env, RcRefCell};
use std::error::Error;
use std::rc::Rc;

static WELCOME_BANNER: &'static str = "Welcome to rx_rs, a simple Scheme interpreter.";

pub fn repl(inport: &mut impl InPort, env: &RcRefCell<Env>) -> Result<(), Box<dyn Error>> {
    let global_env = Rc::clone(env);
    println!("{}", WELCOME_BANNER);
    loop {
        let next_token = inport.next_token();
        match next_token {
            None => (),
            Some(Ok(token_str)) => match inport.read_exp(Some(Ok(token_str))) {
                Ok(exp) => {
                    let val = eval(exp, Rc::clone(&global_env));
                    println!("=> {:?}", val);
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            },
            Some(e) => {
                println!("Readline Error: {:?}", e);
                break;
            }
        }
    }
    Ok(())
}
