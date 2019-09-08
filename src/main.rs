mod eval_apply;
mod parser;
mod prelude;
mod run;
mod types;
use crate::parser::{InFile, InPort, Input};
use std::env;
use std::process;

#[macro_use]
extern crate lazy_static;

fn main() {
    // code goes here
    // println!("Hello, rx_rs!");
    let mut args = env::args();
    let res = match args.nth(1) {
        Some(path) => {
            let mut inport = InFile::new(&path);
            run::repl(&mut inport, true)
        }
        None => {
            let mut inport = Input::new();
            run::repl(&mut inport, false)
        }
    };
    if let Err(e) = res {
        println!("run: {}", e);
        process::exit(1);
    };
}
