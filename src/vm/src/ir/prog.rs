use crate::ir;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Prog<'ir> {
    pub items: BTreeMap<ir::Id, &'ir ir::Item<'ir>>,
}
