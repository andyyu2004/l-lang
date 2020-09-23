use crate::ir::{self, *};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Prog<'ir> {
    pub items: BTreeMap<ir::Id, ir::Item<'ir>>,
    pub impl_items: BTreeMap<ImplItemId, ImplItem<'ir>>,
}
