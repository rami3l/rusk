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
static STDLIB_PATH: &'static str = "./scheme/stdlib.rkt";

fn main() {
    // println!("Hello, rusk!");
    println!("{}", WELCOME_BANNER);
    let mut args = env::args();
    let global_env = make_env_ptr(get_prelude());

    // Interpret source file
    let read_source_file = |path: &str| {
        let mut inport = InFile::new(path);
        print!(".. Reading `{}`: ", inport.file_str);
        run::repl(&mut inport, &mut std::io::sink(), &global_env)
            .expect("Error while loading file.");
        println!("Done.");
    };

    let res = {
        // Load stdlib
        read_source_file(STDLIB_PATH);

        if let Some(path) = args.nth(1) {
            read_source_file(&path);
        };

        // Start REPL Mode
        let mut inport = Input::new();
        run::repl(&mut inport, &mut std::io::stdout(), &global_env)
    };

    if let Err(e) = res {
        eprintln!("run: {}", e);
        process::exit(1);
    };
}
