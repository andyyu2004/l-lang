
fn main() -> number {
    let x = 5;
    let f = fn(y) => x + y;
    let g = fn(y) => x + y;
    apply(f, 8)
}

fn apply(f: fn(number) -> number, x: number) -> number {
    f(x)
}


