extern {
    fn print(i: int);
}


fn main() -> int {
    let s = S::new(5, false);
    print(s.u);
    0
}


struct S<T, U> {
    t: T,
    u: U,
}

impl<T, U> S<T, U> {
    fn new(u: U, t: T) -> Self {
	Self { t, u }
    }
}

impl S<bool, bool> {
    fn new(u: bool, t: bool) -> Self {
	Self { t, u }
    }
}

impl S<int, int> {
    fn new(u: int, t: int) -> Self {
	Self { t, u }
    }
}
