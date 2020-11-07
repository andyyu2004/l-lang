fn main() -> int {
    let ptr = unsafe { &99 };
    take_box(ptr);
    0
}

fn take_box(ptr: &int) {

}
