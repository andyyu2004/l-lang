/* mod fib; */
/* mod dir; */
/* mod test; */
// mod expr;
mod methods;

extern "l-intrinsic" {
    fn print(i: int);
}

fn main() -> int {
    methods::run();
    // run();
    return 0;
}

trait Default {
    fn default() -> Self;
}


