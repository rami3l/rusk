(define range (lambda (a b) (if (= a b) (quote ()) (cons a (range (+ a 1) b)))))

(define fib (lambda (n) (if (< n 2) 1 (+ (fib (- n 1)) (fib (- n 2))))))

(define A (lambda (k x1 x2 x3 x4 x5) 
                    (define B (lambda () 
                        ;; (displayln k)
                        (set! k (- k 1)) (A k B x1 x2 x3 x4)))
                    (if (<= k 0) (+ (x4) (x5)) (B))))

(define (man-or-boy n) (A n (lambda () 1) (lambda () -1) (lambda () -1) (lambda () 1) (lambda () 0)))

(displayln (quote result))
(displayln (man-or-boy 4))

;; (exit)