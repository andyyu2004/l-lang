
struct G<T, U> {
    s: S<U>,
    t: T,
}

struct S<T> {
    t: T
}


fn main() -> int {
    let s: S<_> = S { t: 8 };
    let g = G {
	s, t: false
    };
    g.s.t
}
