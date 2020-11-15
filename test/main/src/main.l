mod fib;
/* mod dir; */

extern {
    fn print(i: int);
}

fn main() -> int {
    /* let x = 5; */
    /* loop { */

    /* print(fib::fib(20)); */
    /* }; */
    /* while x < 10 { */
    /*     print(x); */
    /*     x = x + 1; */
    /* }; */
    print(fib::fib(20));

    return 0;
    /* return 0; */
}


