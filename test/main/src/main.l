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

enum Option<T> {
    Some(T),
    None,
}

fn run() {
    let opt = Option::Some(5);
    let boxed = Option::Some(false);
    match (opt, boxed) {
        (Option::Some(x), Option::Some(false)) => x,
        (Option::Some(x), Option::Some(true)) => x,
        (Option::None, _) => 0,
    };
}


