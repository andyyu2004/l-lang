use super::LoweringCtx;
use crate::ast::*;
use crate::ir;

impl<'ir> LoweringCtx<'ir> {
    pub(super) fn lower_path(&mut self, path: &Path) -> ir::Path<'ir> {
        // just handle the local variable case for now
        todo!()
    }
}
