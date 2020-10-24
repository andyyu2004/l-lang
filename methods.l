extern {
    fn print(i: int);
}

fn main() -> int {
    /* print(S::new().x); */
    let k = Option::Some(5);
    match k {
	Option::Some(x) => x,
	_ => x,
    };
    9
}

enum Option<T> {
    Some(T),
    None,
}

struct S { x: int }

impl S {
    fn new() -> Self {
	Self { x: 5 }
    }
}
