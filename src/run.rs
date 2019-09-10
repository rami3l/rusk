use crate::eval_apply::eval;
use crate::parser::InPort;
use crate::types::{Env, RcRefCellBox};
// use rustyline::{error::ReadlineError, Editor};
// use std::any::{Any, TypeId};
use std::error::Error;
use std::rc::Rc;

static WELCOME_BANNER: &'static str = "Welcome to rx_rs, a simple Scheme interpreter.";

pub fn repl(inport: &mut impl InPort, env: &RcRefCellBox<Env>) -> Result<(), Box<dyn Error>> {
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
                    println!("Error: {:?}", e);
                    continue;
                }
            },
            Some(e) => {
                println!("Readline Error: {:?}", e);
                break;
            } // TODO: fix this break
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{InPort, TOKENIZER};
    use crate::prelude::get_prelude;
    use crate::types::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct MockInput<'a> {
        line: String,
        lines: std::str::Lines<'a>,
    }

    impl<'a> MockInput<'a> {
        fn new(input: &'a str) -> MockInput<'a> {
            MockInput {
                line: String::new(),
                lines: input.lines(),
            }
        }
    }

    impl<'a> InPort for MockInput<'a> {
        fn readline(&mut self) -> Option<Result<String, Box<dyn Error>>> {
            match self.lines.next() {
                Some(line) => (Some(Ok(line.to_string()))),
                None => None,
            }
        }

        fn next_token(&mut self) -> Option<Result<String, Box<dyn Error>>> {
            loop {
                if &self.line == "" {
                    self.line = match self.readline() {
                        Some(Ok(line)) => line,
                        None => String::new(),
                        _ => unreachable!(),
                    };
                }
                if &self.line == "" {
                    return None;
                } else {
                    let next = TOKENIZER.captures_iter(&self.line).next();
                    let (token, rest) = match next {
                        Some(cap) => (String::from(&cap[1]), String::from(&cap[2])),
                        None => unreachable!(),
                    };
                    self.line = rest;
                    match token.chars().nth(0) {
                        Some(';') | None => (),
                        _ => return Some(Ok(token.to_string())),
                    };
                }
            }
        }
    }

    fn check_io_str(input: &str, output: &str, env: &RcRefCellBox<Env>) {
        // let str_exp = input.to_string();
        let mut mock = MockInput::new(input);
        let right = output.to_string();
        let left = match mock.read_next_exp() {
            Ok(exp) => {
                let val = eval(exp, Rc::clone(env));
                format!("{:?}", val)
            }
            Err(e) => format!("Error: {:?}", e),
        };
        assert_eq!(left, right);
    }

    #[test]
    fn plus_simple() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(+ 1 2)", "Ok(3)", &env);
    }

    #[test]
    fn plus_nested() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(+ 1 (* 2 3))", "Ok(7)", &env);
    }

    #[test]
    fn quote() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(quote (1 2 3))", "Ok([1, 2, 3])", &env);
    }

    #[test]
    fn define_val() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(define x 3)", "Ok()", &env);
        check_io_str("x", "Ok(3)", &env);
        check_io_str("(+ x 1)", "Ok(4)", &env);
    }

    #[test]
    fn define_proc_basic() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(define x 3)", "Ok()", &env);
        check_io_str("x", "Ok(3)", &env);
        check_io_str("(define one (lambda () 1))", "Ok()", &env);
        check_io_str("(one)", "Ok(1)", &env);
        check_io_str("(+ (one) (+ 2 x))", "Ok(6)", &env);
    }

    #[test]
    fn define_proc_call_prim() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(define x 3)", "Ok()", &env);
        check_io_str("x", "Ok(3)", &env);
        check_io_str("(define inc (lambda (x) (+ x 1)))", "Ok()", &env);
        check_io_str("(inc 100)", "Ok(101)", &env);
        check_io_str("(inc x)", "Ok(4)", &env);
    }

    #[test]
    fn cond() {
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
    fn eq() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str("(define one (lambda () 1))", "Ok()", &env);
        check_io_str("(= 1 1)", "Ok(true)", &env);
        check_io_str("(= 1 (one))", "Ok(true)", &env);
        check_io_str("(if (= 1 (one)) 123 wtf)", "Ok(123)", &env);
        check_io_str("(if (= (one) (+ 4 5)) wtf 123)", "Ok(123)", &env);
    }

    #[test]
    fn cons() {
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
    fn fibonacci() {
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
            "(map fib (range 0 10))",
            "Ok([1, [1, [2, [3, [5, [8, [13, [21, [34, [55, \
             []]]]]]]]]]])",
            &env,
        );
    }

    #[test]
    #[ignore]
    fn fibonacci_long() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str(
            "(define fib (lambda (n) (if (< n 2) 1 (+ (fib (- n 1)) (fib (- n 2))))))",
            "Ok()",
            &env,
        );
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
            "(map fib (range 0 20))",
            "Ok([1, [1, [2, [3, [5, [8, [13, [21, [34, [55, [89, [144, \
             [233, [377, [610, [987, [1597, [2584, [4181, [6765, \
             []]]]]]]]]]]]]]]]]]]]])",
            &env,
        );
    }

    #[test]
    fn set_simple() {
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
    fn begin() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str(
            "(begin (define one (lambda () 1)) (+ (one) 2))",
            "Ok(3)",
            &env,
        );
    }

    #[test]
    fn multiline_simple() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str(
            "(begin
                (define one
                    (lambda () 1))
                (+ (one) 2))",
            "Ok(3)",
            &env,
        );
    }

    #[test]
    fn multiline_comment() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str(
            "(begin
                (define one
                    ; generating the number 1
                    ;; more quotes
                    (lambda () 1))
                (+ (one) 2))",
            "Ok(3)",
            &env,
        );
    }

    #[test]
    fn define_double_with_begin() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str(
            "(begin
                (define three
                    (begin
                        (define one
                            (lambda () 1))
                        (+ (one) 2)))
                three)",
            "Ok(3)",
            &env,
        );
    }

    #[test]
    fn set_bank_account() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str(
            "(define account
                (lambda (bal)
                    (lambda (amt)
                        (begin 
                            (set! bal (+ bal amt)) 
                            bal))))",
            "Ok()",
            &env,
        );
        check_io_str("(define a1 (account 100))", "Ok()", &env);
        check_io_str("(a1 0)", "Ok(100)", &env);
        check_io_str("(a1 10)", "Ok(110)", &env);
        check_io_str("(a1 10)", "Ok(120)", &env);
    }

    #[test]
    fn lambda() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str(
            "((lambda (x y z)
                (+ x
                   (+ y z))) 1
                             2
                             3)",
            "Ok(6)",
            &env,
        );
    }

    #[test]
    fn sugar_lambda() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str(
            "((lambda (x y z)
                (quote whatever)
                (+ x
                   (+ y z))) 1
                             2
                             3)",
            "Ok(6)",
            &env,
        );
    }

    #[test]
    fn sugar_define_definition() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str(
            "(define (add3 x y z)
                    (+ x
                       (+ y z)))",
            "Ok()",
            &env,
        );
        check_io_str(
            "(add3 101 
                      102 
                      103))",
            "Ok(306)",
            &env,
        );
    }

    #[test]
    fn sugar_define_body() {
        let env: Env = get_prelude();
        let env = Rc::new(RefCell::new(Box::new(env)));
        check_io_str(
            "(define (three)
                (quote whatever)
                (define one (lambda () 1))
                (+ (one) 2))",
            "Ok()",
            &env,
        );
        check_io_str("(three)", "Ok(3)", &env);
    }
}
