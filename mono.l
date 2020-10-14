fn fst<T, U>(t: T, u: U) -> T { t }
fn snd<T, U>(t: T, u: U) -> U { fst(u, t) }

extern {
    fn rc<T>(t: &T) -> int;
}


fn main() -> int {
    let x = box 5;
    rc(x);
}
