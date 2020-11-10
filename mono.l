
fn fst<T, U>((t, u): (T, U)) -> T { t }

fn snd<T, U>((t, u): (T, U)) -> U { fst((u, t)) }

struct S<T, U> {
    t: T,
    u: U,
}

struct K<T, U> {
    s: S<T, U>
}

enum Option<T> {
    Some(T),
    None,
}


fn main() -> int {
    /* fst((1,false)); */
    snd((1, false));
    0
}
