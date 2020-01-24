mod eval_apply;
mod parser;
mod prelude;
mod run;
mod tests;
mod types;

use crate::parser::{InFile, Input};
use crate::prelude::{get_prelude, make_env_ptr};
use clap::App;
use std::process;

#[macro_use]
extern crate clap;

#[macro_use]
extern crate lazy_static;

static WELCOME_BANNER: &'static str = "Welcome to rusk, a simple Scheme interpreter.";
static STDLIB_PATH: &'static str = "./scheme/stdlib.rkt";

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    // println!("Hello, rusk!");
    println!("{}", WELCOME_BANNER);
    let global_env = make_env_ptr(get_prelude());

    // Interpret source file
    let read_source_file = |path: &str| {
        let mut inport = InFile::new(path);
        run::repl(&mut inport, &mut std::io::sink(), &global_env)
    };

    let read_source_file_verbose = |path: &str| {
        let mut inport = InFile::new(path);
        print!(".. Reading `{}`: ", inport.file_str);
        run::repl(&mut inport, &mut std::io::sink(), &global_env)
            .expect("Error while loading file.");
        println!("Done.");
    };

    // REPL mode
    let run_repl = || {
        let mut inport = Input::new();
        run::repl(&mut inport, &mut std::io::stdout(), &global_env)
    };

    let res = {
        // Load stdlib
        read_source_file_verbose(STDLIB_PATH);

        if let Some(path) = matches.value_of("INPUT") {
            if matches.is_present("repl") {
                read_source_file_verbose(&path);
                run_repl()
            } else {
                read_source_file(&path)
            }
        } else {
            run_repl()
        }
    };

    if let Err(e) = res {
        eprintln!("run: {}", e);
        process::exit(1);
    };
}
