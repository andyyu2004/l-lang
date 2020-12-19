/* mod fib; */
/* mod dir; */
/* mod test; */
// mod expr;
mod methods;

extern "l-intrinsic" {
    fn print(i: int);
}

enum Option<T> {
    Some(T),
    None
}

fn main() -> int {
    methods::run();
    match Option::Some(3) {
        Option::Some(x) => x,
        Option::None => 5,
    };
    // run();
    return 0;
}

trait Default {
    fn default() -> Self;
}

