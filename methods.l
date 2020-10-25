extern {
    fn print(i: int);
}

fn main() -> int {
    S::new();
    9
}

struct S { x: int }

impl S {
    fn new() -> Self {
	Self { x: 5 }
    }
}
