fn main() -> number {
    let x = 5;
    apply(fn(y) => x + y, 5)
}

fn apply(f: fn(number) -> number, x: number) -> number {
    f(x)
}


