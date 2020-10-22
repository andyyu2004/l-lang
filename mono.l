
fn f(s: S) {}
struct S { }

extern {
    fn print(i: int);
}

fn main() -> int {
    let x: int = 9;
    let y = return_dangling();
    print(*y);
    0
};

fn return_dangling() -> *int {
    unsafe { &5 }
}

fn take_box(boxed: &int) {
    print(*boxed);
}

