use itertools::Itertools;

pub fn join<'a, I, T>(xs: &'a I, sep: &str) -> String
where
    T: std::fmt::Display,
    &'a I: IntoIterator<Item = T>,
{
    xs.into_iter().map(|x| x.to_string()).collect_vec().join(sep)
}

pub fn join2<'a, I, T>(xs: I, sep: &str) -> String
where
    T: std::fmt::Display,
    I: IntoIterator<Item = T>,
{
    xs.into_iter().map(|x| x.to_string()).collect_vec().join(sep)
}
