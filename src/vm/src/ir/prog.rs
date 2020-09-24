use crate::ir::{self, *};
use std::collections::BTreeMap;

/// top level IR ast
#[derive(Debug)]
pub struct Prog<'ir> {
    pub items: BTreeMap<ir::Id, ir::Item<'ir>>,
    pub impl_items: BTreeMap<ImplItemId, ImplItem<'ir>>,
}
