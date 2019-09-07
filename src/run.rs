use crate::eval_apply::eval;
use crate::parser::parse;
use crate::parser::InPort;
use crate::prelude::get_prelude;
use rustyline::{error::ReadlineError, Editor};
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;

pub fn repl() -> Result<(), Box<dyn Error>> {
    let mut count = 0;
    let global_env = Rc::new(RefCell::new(Box::new(get_prelude())));
    let mut editor = Editor::<()>::new();
    println!("<rx.rs>");
    loop {
        count += 1;
        let readline = editor.readline(&format!("#;{}> ", count));
        match readline {
            Ok(line) => match line.as_ref() {
                ",q" => {
                    println!("Quit");
                    break;
                }
                _ => match parse(&line) {
                    Ok(exp) => {
                        let val = eval(exp, Rc::clone(&global_env));
                        println!("=> {:?}", val);
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                        continue;
                    }
                },
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => return Err(Box::new(err)),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn check_io_str(input: &str, output: &str, env: &RcRefCellBox<Env>) {
        let str_exp = input.to_string();
        let right = output.to_string();
        let left = match parse(&str_exp) {
            Ok(exp) => {
                let val = eval(exp, Rc::clone(env));
                format!("{:?}", val)
            }
            Err(e) => format!("Error: {:?}", e),
        };
        assert_eq!(left, right);
    }

    #[test]
    fn test_plus() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(+ 1 2)", "Ok(3)", &env);
    }

    #[test]
    fn test_plus_nested() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(+ 1 (* 2 3))", "Ok(7)", &env);
    }

    #[test]
    fn test_quote() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(quote (1 2 3))", "Ok([1, 2, 3])", &env);
    }

    #[test]
    fn test_define_val() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(define x 3)", "Ok()", &env);
        check_io_str("x", "Ok(3)", &env);
        check_io_str("(+ x 1)", "Ok(4)", &env);
    }

    #[test]
    fn test_define_proc_basic() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(define x 3)", "Ok()", &env);
        check_io_str("x", "Ok(3)", &env);
        check_io_str("(define one (lambda () 1))", "Ok()", &env);
        check_io_str("(one)", "Ok(1)", &env);
        check_io_str("(+ (one) (+ 2 x))", "Ok(6)", &env);
    }

    #[test]
    fn test_define_proc_call_prim() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(define x 3)", "Ok()", &env);
        check_io_str("x", "Ok(3)", &env);
        check_io_str("(define inc (lambda (x) (+ x 1)))", "Ok()", &env);
        check_io_str("(inc 100)", "Ok(101)", &env);
        check_io_str("(inc x)", "Ok(4)", &env);
    }

    #[test]
    fn test_cond() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(if #t 123 wtf)", "Ok(123)", &env);
        check_io_str("(if #f wtf 123)", "Ok(123)", &env);
        check_io_str(
            "(cond (#f wtf0) (#f wtf1) (#t 456) (else wtf3))",
            "Ok(456)",
            &env,
        );
        check_io_str(
            "(cond (#f wtf0) (#f wtf1) (#f wtf2) (else 789))",
            "Ok(789)",
            &env,
        );
    }

    #[test]
    fn test_eq() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(define one (lambda () 1))", "Ok()", &env);
        check_io_str("(= 1 1)", "Ok(true)", &env);
        check_io_str("(= 1 (one))", "Ok(true)", &env);
        check_io_str("(if (= 1 (one)) 123 wtf)", "Ok(123)", &env);
        check_io_str("(if (= (one) (+ 4 5)) wtf 123)", "Ok(123)", &env);
    }

    #[test]
    fn test_cons() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(car (cons 123 456))", "Ok(123)", &env);
        check_io_str("(cdr (cons 123 456))", "Ok(456)", &env);
        check_io_str("(define p (cons (cons 1 2) (cons 3 4)))", "Ok()", &env);
        check_io_str("(cdr (car p))", "Ok(2)", &env);
        check_io_str("(cdr p)", "Ok([3, 4])", &env);
        check_io_str("p", "Ok([[1, 2], [3, 4]])", &env);
        check_io_str("(define l (cons 1 (cons 2 (cons 3 null))))", "Ok()", &env);
        check_io_str("(car (cdr l))", "Ok(2)", &env);
        check_io_str("(cdr (cdr (cdr l)))", "Ok([])", &env);
    }

    #[test]
    fn test_fibonacci() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str(
            "(define fib (lambda (n) (if (< n 2) 1 (+ (fib (- n 1)) (fib (- n 2))))))",
            "Ok()",
            &env,
        );
        check_io_str("(fib 20)", "Ok(10946)", &env);
        check_io_str(
            "(define range (lambda (a b) (if (= a b) (quote ()) (cons a (range (+ a 1) b)))))",
            "Ok()",
            &env,
        );
        check_io_str(
            "(define map (lambda (f l) (if (null? l) null (cons (f (car l)) (map f (cdr l))))))",
            "Ok()",
            &env,
        );
        check_io_str(
            "(range 0 10)",
            "Ok([0, [1, [2, [3, [4, [5, [6, [7, [8, [9, []]]]]]]]]]])",
            &env,
        );
        check_io_str(
            "(map fib (range 0 20))",
            "Ok([1, [1, [2, [3, [5, [8, [13, [21, [34, [55, [89, [144, \
             [233, [377, [610, [987, [1597, [2584, [4181, [6765, \
             []]]]]]]]]]]]]]]]]]]]])",
            &env,
        );
    }

    #[test]
    fn test_set() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(define inc (lambda (x) (+ x 1)))", "Ok()", &env);
        check_io_str("(define x 3)", "Ok()", &env);
        check_io_str("(set! x (inc x))", "Ok()", &env);
        check_io_str("x", "Ok(4)", &env);
        check_io_str("(set! x (inc x))", "Ok()", &env);
        check_io_str("x", "Ok(5)", &env);
    }

    #[test]
    fn test_begin() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str(
            "(begin (define one (lambda () 1)) (+ (one) 2))",
            "Ok(3)",
            &env,
        );
    }
}
