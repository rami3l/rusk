mod eval_apply;
mod parser;
mod prelude;
mod run;
mod types;
use std::process;

#[macro_use]
extern crate lazy_static;

fn main() {
    // code goes here
    // println!("Hello, rx_rs!");
    if let Err(e) = run::repl() {
        println!("run: {}", e);
        process::exit(1);
    };
}
