use crate::ir;
use std::collections::BTreeMap;

#[derive(Debug)]
crate struct Prog<'ir> {
    pub items: BTreeMap<ir::Id, &'ir ir::Item<'ir>>,
}
