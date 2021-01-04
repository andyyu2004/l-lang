
fn main() -> int { 0 }

enum Enum<T> {
}

trait SomeTrait {}

impl SomeTrait for Enum {} //~ ERROR expected 1 generic parameter but received 0
