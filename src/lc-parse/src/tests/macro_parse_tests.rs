use super::*;

#[test]
fn parse_macro_def() {
    parse_macro!({
        ($($tt:tt)*, random_thing $($expr:expr):*) => {
            $($tt)* : $($expr)*
        }
    });
}
