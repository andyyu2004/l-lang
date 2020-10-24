extern {
    fn print(i: int);
}

fn main() -> int {
    print(new().x);
    9
}

struct S { x: int }

impl S {
    fn new() -> Self {
	Self { x: 5 }
    }
}
