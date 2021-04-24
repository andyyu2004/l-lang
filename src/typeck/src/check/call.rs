use super::FnCtx;
use lcore::ty::{self, *};

enum Callstep<'tcx> {
    Fn(Ty<'tcx>),
    Closure(Ty<'tcx>),
    // bit of a work around to not require type annotations in certain places
    // `fn(p) => p(3)` would fail if we were to `partially_resolve_ty`
    Infer,
}

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub(super) fn check_call_expr(
        &mut self,
        expr: &ir::Expr<'tcx>,
        f: &ir::Expr<'tcx>,
        args: &[ir::Expr<'tcx>],
    ) -> Ty<'tcx> {
        let ret = self.new_infer_var(expr.span);
        let params = self.check_expr_list(args);

        let initial_fty = self.check_expr(f);
        let callstep = self.try_callstep(initial_fty);
        let fty = match callstep {
            Some(Callstep::Fn(ty)) => ty,
            Some(Callstep::Closure(ty)) => ty,
            Some(Callstep::Infer) => initial_fty,
            None => return self.emit_ty_err(expr.span, TypeError::NonFunctionCall(initial_fty)),
        };
        let ty = self.tcx.mk_fn_ptr(FnSig { params, ret });
        self.unify(expr.span, ty, fty);
        ret
    }

    fn try_callstep(&mut self, fty: Ty<'tcx>) -> Option<Callstep<'tcx>> {
        match fty.kind {
            ty::FnPtr(..) => Some(Callstep::Fn(fty)),
            ty::Closure(sig) => Some(Callstep::Closure(self.mk_fn_ptr(sig))),
            ty::Infer(..) => Some(Callstep::Infer),
            _ => None,
        }
    }

    pub(super) fn check_method_call_expr(
        &mut self,
        expr: &ir::Expr<'tcx>,
        segment: &ir::PathSegment<'tcx>,
        args: &[ir::Expr<'tcx>],
    ) -> Ty<'tcx> {
        let (receiver, args) = args.split_first().unwrap();
        let receiver_ty = self.check_expr(receiver);
        let res = self.resolve_method(expr, receiver_ty, segment);
        todo!()
    }
}
