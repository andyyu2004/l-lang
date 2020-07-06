use super::Item;

/// top level ast representation that stores entire contents of the program being compiled
#[derive(Debug)]
crate struct Prog {
    pub items: Vec<Item>,
}
