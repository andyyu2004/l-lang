
fn f(s: S) {}
struct S { }

extern {
    fn print(i: int);
}

fn main() -> int {
    let x: int = *(box 5);
    let y: int = unsafe { *&x };
};

fn take_box(boxed: &int) {
    print(*boxed);
}

