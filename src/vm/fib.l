fn main() -> int {
    /* fn f(n) { */
    /*     if n < 2 { n } else { f(n-1) + f(n-2) } */
    /* }; */
    /* fn () => 5; */
    /* f(5); */
    *boxed_fib(40)
}

fn fac(n: int) -> int {
    if n < 1 { 1 } else { n * fac(n-1) }
}

fn fib(n: int) -> int {
    if n < 2 { n } else { fib(n-1) + fib(n-2) }
}

fn boxed_fib(n: int) -> &int {
    if n < 2 { box n } else { box (*boxed_fib(n-1) + *boxed_fib(n-2)) }
}
