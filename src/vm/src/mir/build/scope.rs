use super::{BlockAnd, BlockId, Builder, VarId};
use crate::mir::{SpanInfo, Stmt, StmtKind};
use crate::set;

#[derive(Default, Debug)]
crate struct Scopes {
    scopes: Vec<Scope>,
}

impl Scopes {
    fn pop_scope(&mut self) -> Scope {
        self.scopes.pop().unwrap()
    }

    fn push_scope(&mut self, scope: Scope) {
        self.scopes.push(scope)
    }
}

#[derive(Debug)]
struct Release {
    var: VarId,
    info: SpanInfo,
}

#[derive(Default, Debug)]
struct Scope {
    /// list of variables to be `release`d at the end of the scope
    releases: Vec<Release>,
}

impl<'a, 'tcx> Builder<'a, 'tcx> {
    fn exit_scope(&mut self, info: SpanInfo, block: BlockId) {
        let scope = self.scopes.pop_scope();
        for release in scope.releases {
            self.push(block, Stmt { info, kind: StmtKind::Release(release.var) })
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
