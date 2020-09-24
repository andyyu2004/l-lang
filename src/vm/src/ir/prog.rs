use crate::ir::{self, *};
use std::collections::BTreeMap;

/// top level IR ast
#[derive(Debug)]
pub struct Prog<'ir> {
    /// `Id` of the entry/main function
    pub entry_id: Option<DefId>,
    pub items: BTreeMap<DefId, ir::Item<'ir>>,
    pub impl_items: BTreeMap<ImplItemId, ImplItem<'ir>>,
}
