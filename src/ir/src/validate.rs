use crate::{self as ir, DefId, Visitor};

pub struct Validator<'ir> {
    ir: &'ir ir::Ir<'ir>,
    curr_def_id: Option<DefId>,
}

impl<'ir> Validator<'ir> {
    pub fn new(ir: &'ir ir::Ir<'ir>) -> Self {
        Self { ir, curr_def_id: None }
    }
}

impl<'ir> Visitor<'ir> for Validator<'ir> {
    fn visit_item(&mut self, item: &'ir ir::Item<'ir>) {
        self.curr_def_id = Some(item.id.def);
        ir::walk_item(self, item);
    }
}
