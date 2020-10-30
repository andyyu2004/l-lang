// extern {
//     fn print(i: int);
// }


fn main() -> int {
    let s = S::new(5, false);
    s.u
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

impl S<int, int> {
    fn new(u: int, t: int) -> Self {
	Self { t, u }
    }
}
