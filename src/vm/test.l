
enum Option<T> {
    Some(T),
    None,
}

enum Either<L, R> {
    Left(L),
    Right(R),
}

struct Complex<T, U> {
    either: Either<U, T>,
    option: Option<U>,
}

fn main() -> int {
    let option: Option<_> = Option::Some(false);
    let either: Either<_, _> = Either::Right(9);

    Complex { either, option };
    5
}
