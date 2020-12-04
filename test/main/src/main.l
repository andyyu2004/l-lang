/* mod fib; */
/* mod dir; */
/* mod test; */
// mod expr;
mod methods;

extern {
    fn print(i: int);
}

fn main() -> int {
    methods::run();
    return 0;
}


