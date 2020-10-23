
fn f(s: S) {}

enum Option<T> {
    Some(T),
    None,
}

struct S {
    b: bool,
    x: int,
    y: int,
}

fn main() -> int {
    let s = S {
	x: 9,
	b: false,
	y: 7,
    };

    let S { x, y, x: i } = s;
    x - y
}
