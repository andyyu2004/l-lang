fn main() -> number {
    return 5;
    return 4;
    0
}

fn is_odd(n: number) -> bool {
    if (n < 1) { false } else { is_even(n - 1) }
}

fn is_even(n: number) -> bool {
    if (n < 1) { true } else { is_odd(n - 1) }
}







