extern {
    fn print(i: int);
}

fn main() -> int {
    print(S::fuck());
    9
}

struct S {
    x: int
}

impl S {
    fn fuck() -> int {
	5
    }
}
