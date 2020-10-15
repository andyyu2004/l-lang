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

enum MonoOption {
    Some(int),
    None
}

fn main() -> int {
    let opt = Option::Some(8);
    match opt {
	Option::Some(k) => k,
    }
}
