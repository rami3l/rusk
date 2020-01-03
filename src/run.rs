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
            None => break,
            Some(Ok(token_str)) => match inport.read_exp(Some(Ok(token_str))) {
                Ok(exp) => {
                    let val = eval(exp, Rc::clone(&global_env));
                    writeln!(outport, "=> {:?}", val)?;
                }
                Err(e) => {
                    writeln!(outport, "Error: {}", e)?;
                }
            },
            Some(e) => {
                writeln!(outport, "Readline Error: {:?}", e)?;
                break;
            }
        }
    }
    Ok(())
}
