// extern {
//     fn print(i: int);
// }


fn main() -> int {
    S::new(5)
}


struct S<T> {
    t: T,
}

impl<T> S<T> {
    fn new(t: T) -> S<T> {
	Self { t }
    }
}
