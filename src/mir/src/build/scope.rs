use super::*;
use crate::set;
use std::marker::PhantomData;

#[derive(Default, Debug)]
crate struct Scopes<'tcx> {
    scopes: Vec<Scope<'tcx>>,
    breakable_scopes: Vec<BreakableScope<'tcx>>,
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

    fn peek_breakable(&self) -> &BreakableScope<'tcx> {
        self.breakable_scopes.last().unwrap()
    }

    fn peek_mut(&mut self) -> &mut Scope<'tcx> {
        self.scopes.last_mut().unwrap()
    }
}

#[derive(Debug)]
crate struct ReleaseInfo {
    pub info: SpanInfo,
    pub var: VarId,
}

#[derive(Default, Debug)]
struct Scope<'tcx> {
    /// list of variables to be `release`d at the end of the scope
    releases: Vec<ReleaseInfo>,
    pd: PhantomData<&'tcx ()>,
}

#[derive(Debug)]
struct BreakableScope<'tcx> {
    /// the block to branch to on continue
    /// typically the start of the loop
    continue_block: BlockId,
    /// the block to branch to on break
    break_block: BlockId,
    /// the lvalue to write the break expression to
    /// only available for `loops` (not `for` or `while` loops)
    lvalue: Option<Lvalue<'tcx>>,
}

#[derive(Debug)]
crate enum BreakType {
    Continue,
    Break,
}

impl<'a, 'tcx> MirBuilder<'a, 'tcx> {
    pub fn schedule_release(&mut self, info: SpanInfo, var: VarId) {
        let scope = self.scopes.peek_mut();
        scope.releases.push(ReleaseInfo { var, info });
    }

    fn exit_scope(&mut self, _info: SpanInfo, block: BlockId) {
        let scope = self.scopes.pop_scope();
        for release in scope.releases.into_iter().rev() {
            self.push_release(block, release);
        }
    }

    pub fn break_scope(&mut self, info: SpanInfo, block: BlockId, kind: BreakType) -> BlockAnd<()> {
        match kind {
            BreakType::Continue => {
                let continue_block = self.scopes.peek_breakable().continue_block;
                self.branch(info, block, continue_block);
            }
            BreakType::Break => {
                let break_block = self.scopes.peek_breakable().break_block;
                self.branch(info, block, break_block);
            }
        }
        // return some new unreachable block to write the unreachable stuff into
        self.append_basic_block().unit()
    }

    /// `block` is the block where a `break` expr should branch to
    /// the function returns the `BlockAnd` where normal execution should go
    pub fn with_breakable_scope(
        &mut self,
        span: Span,
        continue_block: BlockId,
        break_block: BlockId,
        f: impl FnOnce(&mut Self) -> BlockAnd<()>,
    ) -> BlockAnd<()> {
        let info = self.span_info(span);
        self.scopes.breakable_scopes.push(BreakableScope {
            break_block,
            continue_block,
            lvalue: None,
        });
        let normal_block = f(self).block;
        self.scopes.breakable_scopes.pop();

        let new = self.append_basic_block();
        self.branch(info, normal_block, new);
        self.branch(info, break_block, new);
        new.unit()
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
