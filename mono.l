fn fst<T, U>(t: T, u: U) -> T { t }
fn snd<T, U>(t: T, u: U) -> U { fst(u, t) }

extern {
    fn rc<T>(t: &T) -> int;
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
    let opt = MonoOption::Some(8);
    match opt {
	MonoOption::Some(k) => k,
    }
}
