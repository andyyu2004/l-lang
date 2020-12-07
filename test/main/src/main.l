/* mod fib; */
/* mod dir; */
/* mod test; */
// mod expr;
mod methods;

extern  {
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
    let p = Option::Some(false);
    let q = Option::Some(5);
    match false {
        true => false,
        false => true,
    };


    match (p, q) {
        (Option::None, Option::None) => false,
        (_, Option::Some(_)) => true,
        (Option::Some(_), Option::None) => true,
        (Option::Some(_), Option::Some(_)) => false,
    };

    let p = false;
    let q = true;
    match (p, q) {
        (false, false) => false,
        (false, true) => true,
        (true, false) => true,
        (true, true) => false,
    };

    let p = Option::Some(false);
    let q = box Option::Some(5);
    match q {
        &Option::None => false,
        &Option::Some(_) => false,
    };
    match (p, q) {
        (Option::None, &Option::None) => false,
        (Option::None, &Option::Some(_)) => true,
        (Option::Some(_), &Option::None) => true,
        (Option::Some(_), &Option::Some(_)) => false,
    };

    // match (p, q) {
    //     (Option::None, &Option::None) => false,
    //     (Option::None, &Option::Some(_)) => true,
    //     (Option::Some(_), &Option::None) => true,
    //     (Option::Some(_), &Option::Some(_)) => false,
    // };
}
