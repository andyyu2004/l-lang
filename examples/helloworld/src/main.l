/* mod fib; */
/* mod dir; */
/* mod test; */
// mod expr;
mod gc;
mod methods;

extern "l-intrinsic" {
    fn print(i: int);
}

enum Option<T> {
    Some(T),
    None
}

fn main() -> int {
    let mut i = 0;
    loop {
        if i > 50000 { break }
        i = i + 1;
        let boxed = box i;
        print(*boxed);
    }
    methods::run();
    gc::run();
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


enum Enum<T> {
}
