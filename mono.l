fn fst<T, U>(t: T, u: U) -> T { t }
fn snd<T, U>(t: T, u: U) -> U { fst(u, t) }


fn main() -> int {
    let x = 5;
    if snd(x, true) {
	5
    } else {
	8
    }
}
