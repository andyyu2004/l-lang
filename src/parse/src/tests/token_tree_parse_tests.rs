use super::*;

macro_rules! parse_token_tree {
    ($src:tt) => {{
        let s = stringify!($src);
        let mut chars = s.chars();
        assert_eq!(chars.next().unwrap(), '{');
        assert_eq!(chars.next_back().unwrap(), '}');
        let driver = ldriver::Driver::from_src(chars.as_str());
        driver.parse_tt().unwrap()
    }};
}

#[test]
fn test_parse_simple_token_tree() {
    println!("");
    let stream = parse_token_tree!({
        fn main() {
        }
    });
    dbg!(stream);
}
