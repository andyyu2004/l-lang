use crate::FnCtx;
use ir::{QPath, Res};

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn resolve_qpath(&mut self, qpath: &QPath) -> Res {
        match qpath {
            QPath::Resolved(path) => path.res,
            QPath::TypeRelative(_, _) => todo!(),
        }
    }
}
