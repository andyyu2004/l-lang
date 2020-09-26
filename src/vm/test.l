
enum Option<T> {
    Some(T),
    None,
}

fn main() -> int {
    let opt: Option<_> = Option::Some(5);
    5
}
