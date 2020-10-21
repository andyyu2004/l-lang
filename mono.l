
fn f(s: S) {}
struct S { }

extern {
    fn print(i: int);
}

fn main() -> int {
    let x = 5;
    let boxed = box 5;
    take_box(boxed);
    unsafe { print(*&x) };
    0
}

fn take_box(boxed: &int) {
    print(*boxed);
}

