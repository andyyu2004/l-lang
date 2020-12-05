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
    let p = false;
    let q = true;
    match (p, q) {
        (false, false) => false,
        (false, true) => true,
        (true, false) => true,
        (true, true) => false,
    };
}


