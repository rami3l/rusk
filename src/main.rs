mod eval_apply;
mod parser;
mod prelude;
mod run;
mod types;

use crate::parser::{InFile, Input};
use crate::prelude::{get_prelude, make_env_ptr};
use std::env;
use std::process;

#[macro_use]
extern crate lazy_static;

fn main() {
    // println!("Hello, rx_rs!");
    let mut args = env::args();
    let global_env = make_env_ptr(get_prelude());
    let res = match args.nth(1) {
        Some(path) => {
            // * Interpret source file
            let mut inport = InFile::new(&path);
            println!("Reading file \"{}\"", inport.file_str);
            run::repl(&mut inport, &global_env)
        }
        None => {
            // * REPL Mode
            let mut inport = Input::new();
            run::repl(&mut inport, &global_env)
        }
    };
    if let Err(e) = res {
        println!("run: {}", e);
        process::exit(1);
    };
}
