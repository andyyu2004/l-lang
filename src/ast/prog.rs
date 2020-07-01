use super::Item;

/// top level ast representation that stores entire contents of the program being compiled
pub struct Prog {
    items: Vec<Item>,
}
