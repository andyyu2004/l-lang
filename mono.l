fn fst<T, U>(t: T, u: U) -> T { t }
fn snd<T, U>(t: T, u: U) -> U { fst(u, t) }

extern {
    fn rc<T>(t: &T) -> int;
}


struct S<T> {
    t: T
}

enum Option<T> {
    Some(T),
    None,
}

enum Either<L, R> {
    Left(L),
    Right(R),
}

fn main() -> int {
    Option::Some(8);
    /* let x = snd(false, 9); */
    /* snd(9, false); */
    snd(9, false);
    snd(false, 9);
    8
}
