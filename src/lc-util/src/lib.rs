use itertools::Itertools;

pub fn join<'a, I, T>(xs: &'a I, sep: &str) -> String
where
    T: std::fmt::Display,
    &'a I: IntoIterator<Item = T>,
{
    xs.into_iter().map(|x| x.to_string()).collect_vec().join(sep)
}

pub fn join2<I, T>(xs: I, sep: &str) -> String
where
    T: std::fmt::Display,
    I: IntoIterator<Item = T>,
{
    xs.into_iter().map(|x| x.to_string()).collect_vec().join(sep)
}

#[macro_export]
macro_rules! pluralize {
    ($x:expr) => {
        if $x != 1 { "s" } else { "" }
    };
}

#[macro_export]
macro_rules! stringify_tt {
    ($tt:tt) => {{
        let s = stringify!($tt);
        let mut chars = s.chars();
        assert_eq!(chars.next().unwrap(), '{');
        assert_eq!(chars.next_back().unwrap(), '}');
        chars.as_str()
    }};
}
