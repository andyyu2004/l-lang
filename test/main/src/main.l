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

fn run() -> int {
    let opt = box Option::None;
    match opt {
        &Option::Some(x) => x,
        // &Option::None => x,
    }
}


