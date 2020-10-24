extern {
    fn print(i: int);
}

fn main() -> int {
    print(new());
    9
}

struct S;

impl S {
    fn new() -> int {
	9
    }
}
