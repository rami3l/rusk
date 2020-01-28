use crate::eval_apply::eval;
use crate::parser::InPort;
use crate::types::{Env, RcRefCell};
use std::rc::Rc;

pub fn repl(
    inport: &mut impl InPort,
    outport: &mut impl std::io::Write,
    env: &RcRefCell<Env>,
) -> Result<(), std::io::Error> {
    let global_env = Rc::clone(env);
    loop {
        let next_token = inport.next_token();
        match next_token {
            Ok(None) => break,
            Ok(Some(token_str)) => match inport.read_exp(Ok(Some(token_str))) {
                Ok(exp) => {
                    let val = eval(exp, Rc::clone(&global_env));
                    writeln!(outport, "=> {:?}", val)?;
                }
                Err(e) => {
                    writeln!(outport, "Error: {:?}", e)?;
                }
            },
            Err(e) => {
                eprintln!("Readline Error: {:?}", e);
                break;
            }
        }
    }
    Ok(())
}
