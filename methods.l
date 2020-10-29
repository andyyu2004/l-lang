extern {
    fn print(i: int);
}


fn main() -> int {
    S::new(5);
    9
}


struct S<T> {
    t: T,
}

impl<T> S<T> {
    fn new(t: T) -> Self {
	Self { t }
    }
}
