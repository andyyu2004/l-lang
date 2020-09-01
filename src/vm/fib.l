fn main() -> number {
    /* fn f(n) { */
    /*     if n < 2 { n } else { f(n-1) + f(n-2) } */
    /* }; */
    /* fn () => 5; */
    /* f(5); */
    fib(2)
}

fn fib(n: number) -> number {
    if n < 1 { 1 } else { 2 }
    /* if n < 2 { n } else { fib(n-1) + fib(n-2) } */
}
