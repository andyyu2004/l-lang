mod fib;
mod dir;

extern {
    fn print(i: int);
}

fn main() -> int {
    print(fib::fib(20));
    0
}

