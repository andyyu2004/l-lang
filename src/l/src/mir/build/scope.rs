use super::{BlockAnd, BlockId, Builder, VarId};
use crate::mir::Lvalue;
use crate::mir::{SpanInfo, Stmt, StmtKind};
use crate::set;

#[derive(Default, Debug)]
crate struct Scopes<'tcx> {
    scopes: Vec<Scope<'tcx>>,
}

impl<'tcx> Scopes<'tcx> {
    fn pop_scope(&mut self) -> Scope<'tcx> {
        self.scopes.pop().unwrap()
    }

    fn push_scope(&mut self, scope: Scope<'tcx>) {
        self.scopes.push(scope)
    }

    fn peek(&self) -> &Scope<'tcx> {
        self.scopes.last().unwrap()
    }

    fn peek_mut(&mut self) -> &mut Scope<'tcx> {
        self.scopes.last_mut().unwrap()
    }
}

#[derive(Debug)]
crate struct ReleaseInfo<'tcx> {
    pub info: SpanInfo,
    pub lvalue: Lvalue<'tcx>,
}

#[derive(Default, Debug)]
struct Scope<'tcx> {
    /// list of variables to be `release`d at the end of the scope
    releases: Vec<ReleaseInfo<'tcx>>,
}

impl<'a, 'tcx> Builder<'a, 'tcx> {
    pub fn schedule_release(&mut self, info: SpanInfo, lvalue: Lvalue<'tcx>) {
        let scope = self.scopes.peek_mut();
        scope.releases.push(ReleaseInfo { lvalue, info });
    }

    fn exit_scope(&mut self, info: SpanInfo, block: BlockId) {
        let scope = self.scopes.pop_scope();
        for release in scope.releases {
            self.push_release(block, release);
        }
    }

    pub fn with_scope<R>(
        &mut self,
        info: SpanInfo,
        f: impl FnOnce(&mut Self) -> BlockAnd<R>,
    ) -> BlockAnd<R> {
        self.scopes.push_scope(Scope::default());
        let block;
        let ret = set!(block = f(self));
        self.exit_scope(info, block);
        block.and(ret)
    }
}
