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

enum Option {
    Some(int),
    None,
}

fn run() -> int {
    let opt = Option::None;
    match opt {
        Option::None => 5,
        Option::Some(x) => 55,
    }
}


