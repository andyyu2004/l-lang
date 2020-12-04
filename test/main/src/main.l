/* mod fib; */
/* mod dir; */
/* mod test; */
// mod expr;
mod methods;

extern {
    fn print(i: int);
}

// fn main() -> int {
//     methods::run();
//     return 0;
// }
enum Option {
    Some(int),
    None,
}

fn main1() -> int {
    let opt = Option::None;
    match opt {
        Option::Some(x) => x,
        Option::None() => 99,
    }
}


enum Option2 {
    Some(int),
    None(),
}

fn main2() -> int {
    let opt = Option2::None();
    match opt {
        Option2::Some(x) => x,
        Option2::None => 99,
    }
}
