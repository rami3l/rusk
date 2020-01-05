mod eval_apply;
mod parser;
mod prelude;
mod run;
mod tests;
mod types;

use crate::parser::{InFile, Input};
use crate::prelude::{get_prelude, make_env_ptr};
use std::env;
use std::process;

#[macro_use]
extern crate lazy_static;

static WELCOME_BANNER: &'static str = "Welcome to rusk, a simple Scheme interpreter.";

fn main() {
    // println!("Hello, rusk!");
    println!("{}", WELCOME_BANNER);
    let mut args = env::args();
    let global_env = make_env_ptr(get_prelude());
    let res = match args.nth(1) {
        Some(path) => {
            // * Interpret source file
            let mut inport = InFile::new(&path);
            println!("rusk: Reading file \"{}\" ...", inport.file_str);
            run::repl(&mut inport, &mut std::io::sink(), &global_env)
                .expect("Error while loading file.");
            println!("rusk: Source file loaded successfully.");
            let mut inport = Input::new();
            run::repl(&mut inport, &mut std::io::stdout(), &global_env)
        }
        None => {
            // * REPL Mode
            let mut inport = Input::new();
            run::repl(&mut inport, &mut std::io::stdout(), &global_env)
        }
    };
    if let Err(e) = res {
        println!("run: {}", e);
        process::exit(1);
    };
}
