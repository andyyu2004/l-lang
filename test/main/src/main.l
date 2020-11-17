/* mod fib; */
/* mod dir; */
mod test;

extern {
    fn print(i: int);
}

fn main() -> int {
    test::uninit();
    return 0;
    let x = 0;
    loop {
	x = x + 1;
	if x < 5 {
	    continue;
	}
	if x > 10 {
	    break;
	}
	print(x);
    }
    /* while x < 10 { */
    /*     print(x); */
    /*     x = x + 1; */
    /* }; */
    /* print(fib::fib(20)); */

    return 0;
    /* return 0; */
}


