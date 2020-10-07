use super::FnCtx;
use crate::{Autoderef, TyConv, Typeof};
use ast::{BinOp, Lit};
use ast::{Ident, Mutability, UnaryOp};
use ir::{CtorKind, DefId, DefKind};
use itertools::Itertools;
use lcore::ty::*;
use rustc_hash::FxHashMap;
use span::Span;

impl<'a, 'tcx> FnCtx<'a, 'tcx> {
    pub fn check_expr(&mut self, expr: &ir::Expr) -> Ty<'tcx> {
        let ty = match &expr.kind {
            ir::ExprKind::Lit(lit) => self.check_lit(lit),
            ir::ExprKind::Bin(op, l, r) => self.check_binop(*op, l, r),
            ir::ExprKind::Unary(op, operand) => self.check_unary_expr(expr, *op, operand),
            ir::ExprKind::Block(block) => self.check_block(block),
            ir::ExprKind::Path(path) => self.check_expr_path(path),
            ir::ExprKind::Tuple(xs) => self.check_expr_tuple(xs),
            ir::ExprKind::Closure(sig, body) => self.check_closure_expr(expr, sig, body),
            ir::ExprKind::Call(f, args) => self.check_call_expr(expr, f, args),
            ir::ExprKind::Match(expr, arms, src) => self.check_match_expr(expr, arms, src),
            ir::ExprKind::Struct(path, fields) => self.check_struct_expr(expr, path, fields),
            ir::ExprKind::Assign(l, r) => self.check_assign_expr(expr, l, r),
            ir::ExprKind::Ret(ret) => self.check_ret_expr(expr, ret.as_deref()),
            ir::ExprKind::Field(base, ident) => self.check_field_expr(expr, base, *ident),
            ir::ExprKind::Box(expr) => self.check_box_expr(expr),
        };
        self.write_ty(expr.id, ty)
    }

    fn check_unary_expr(&mut self, expr: &ir::Expr, op: UnaryOp, operand: &ir::Expr) -> Ty<'tcx> {
        let operand_ty = self.check_expr(operand);
        match op {
            UnaryOp::Neg => todo!(),
            UnaryOp::Not => {
                self.unify(expr.span, self.types.boolean, operand_ty);
                self.types.boolean
            }
            // TODO how to handle mutability?
            UnaryOp::Deref => {
                let ty = self.new_infer_var(expr.span);
                self.unify(expr.span, self.mk_ptr_ty(Mutability::Mut, ty), operand_ty);
                ty
            }
            UnaryOp::Ref => self.mk_ptr_ty(Mutability::Mut, operand_ty),
        }
    }

    fn check_box_expr(&mut self, expr: &ir::Expr) -> Ty<'tcx> {
        let ty = self.check_expr(expr);
        // TODO unsure how to treat mutability, just setting to mutable for now
        self.mk_ptr_ty(Mutability::Mut, ty)
    }

    fn check_field_expr(&mut self, expr: &ir::Expr, base: &ir::Expr, ident: Ident) -> Ty<'tcx> {
        let (autoderef, ty) = self.check_field_expr_inner(expr, base, ident);
        let adjustments = autoderef.get_adjustments();
        self.write_adjustments(base.id, adjustments);
        ty
    }

    fn check_field_expr_inner(
        &mut self,
        expr: &ir::Expr,
        base: &ir::Expr,
        ident: Ident,
    ) -> (Autoderef<'_, 'tcx>, Ty<'tcx>) {
        let base_ty = self.check_expr(base);
        let mut autoderef = self.autoderef(expr.span, base_ty);
        for ty in &mut autoderef {
            match ty.kind {
                Adt(adt, substs) if adt.kind != AdtKind::Enum => {
                    let variant = adt.single_variant();
                    let ty = if let Some((idx, field)) =
                        variant.fields.iter().find_position(|f| f.ident == ident)
                    {
                        // note the id belongs is the id of the entire field expression
                        // not just the identifier or base
                        self.write_field_index(expr.id, idx);
                        field.ty(self.tcx, substs)
                    } else {
                        self.emit_ty_err(expr.span, TypeError::UnknownField(base_ty, ident))
                    };
                    return (autoderef, ty);
                }
                Tuple(tys) => {
                    // `tuple.i` literally means the i'th element of tuple
                    // so we can weirdly parse the identifier as the actual index
                    let idx = ident.as_str().parse::<usize>().unwrap();
                    self.write_field_index(expr.id, idx);
                    let ty = match tys.get(idx) {
                        Some(ty) => ty,
                        None =>
                            self.emit_ty_err(expr.span, TypeError::TupleOutOfBounds(idx, tys.len())),
                    };
                    return (autoderef, ty);
                }
                _ => continue,
            }
        }
        panic!("bad field access, todo proper error msg {}", base_ty);
    }

    /// return expressions have the type of the expression that follows the return
    fn check_ret_expr(&mut self, expr: &ir::Expr, ret_expr: Option<&ir::Expr>) -> Ty<'tcx> {
        let ty = ret_expr.map(|expr| self.check_expr(expr)).unwrap_or(self.tcx.types.unit);
        self.unify(expr.span, self.ret_ty, ty);
        self.tcx.types.never
    }

    /// checks the expressions is a lvalue and mutable, hence assignable
    // TODO mutability checks
    fn check_lvalue(&mut self, l: &ir::Expr) {
        if !l.is_lvalue() {
            self.emit_ty_err(
                l.span,
                TypeError::Msg(format!("expected lvalue as target of assignment")),
            );
        }
    }

    fn check_assign_expr(&mut self, expr: &ir::Expr, l: &ir::Expr, r: &ir::Expr) -> Ty<'tcx> {
        self.check_lvalue(l);
        let lty = self.check_expr(l);
        let rty = self.check_expr(r);
        self.unify(expr.span, lty, rty);
        rty
    }

    fn check_struct_path(&mut self, path: &ir::Path) -> Option<(&'tcx VariantTy<'tcx>, Ty<'tcx>)> {
        let ty = self.check_expr_path(path);
        let variant = match path.res {
            ir::Res::Def(_, DefKind::Struct) => match ty.kind {
                Adt(adt, substs) => Some((adt.single_variant(), ty)),
                _ => unreachable!(),
            },
            ir::Res::Def(def_id, DefKind::Ctor(CtorKind::Struct)) => match ty.kind {
                Adt(adt, _substs) => Some((adt.variant_with_ctor(def_id), ty)),
                _ => unreachable!(),
            },
            ir::Res::Local(_) => None,
            ir::Res::PrimTy(_) => unreachable!(),
            _ => unimplemented!("{} (res: {:?})", path, path.res),
        };

        variant.or_else(|| {
            self.emit_ty_err(
                path.span,
                TypeError::Msg(format!("expected struct path, found {:?}", path)),
            );
            None
        })
    }

    fn check_struct_expr(
        &mut self,
        expr: &ir::Expr,
        path: &ir::Path,
        fields: &[ir::Field],
    ) -> Ty<'tcx> {
        let (variant_ty, ty) = match self.check_struct_path(path) {
            Some(tys) => tys,
            None => return self.tcx.mk_ty_err(),
        };
        let (adt_ty, substs) = ty.expect_adt();
        self.check_struct_expr_fields(expr, adt_ty, substs, variant_ty, fields);
        ty
    }

    fn check_struct_expr_fields(
        &mut self,
        expr: &ir::Expr,
        adt_ty: &AdtTy,
        substs: SubstsRef<'tcx>,
        variant: &VariantTy<'tcx>,
        fields: &[ir::Field],
    ) -> bool {
        // note we preserve the field declaration order of the struct
        let mut remaining_fields = variant
            .fields
            .iter()
            .enumerate()
            .map(|(i, f)| (f.ident, (i, f)))
            .collect::<FxHashMap<Ident, (usize, &FieldTy)>>();
        let mut seen_fields = FxHashMap::default();
        let mut has_error = false;
        for field in fields {
            // handle unknown field or setting field twice
            let ty = self.check_expr(field.expr);
            match remaining_fields.remove(&field.ident) {
                Some((idx, f)) => {
                    seen_fields.insert(field.ident, idx);
                    self.write_field_index(field.id, idx);
                    self.unify(field.span, f.ty(self.tcx, substs), ty);
                }
                None => {
                    has_error = true;
                    if let Some(&idx) = seen_fields.get(&field.ident) {
                        // write the index even on error to avoid missing
                        // entries in table later (may be unnecessary)
                        self.write_field_index(field.id, idx);
                        self.emit_ty_err(
                            field.span,
                            TypeError::Msg(format!("field `{}` set more than once", field.ident)),
                        );
                    } else {
                        self.emit_ty_err(
                            field.span,
                            TypeError::Msg(format!("unknown field `{}`", field.ident)),
                        );
                    }
                }
            }
        }

        if !remaining_fields.is_empty() {
            has_error = true;
            self.emit_ty_err(expr.span, TypeError::Msg(format!("incomplete fields")));
        }

        has_error
    }

    fn check_match_expr(
        &mut self,
        expr: &ir::Expr,
        arms: &[ir::Arm],
        src: &ir::MatchSource,
    ) -> Ty<'tcx> {
        let expr_ty = self.check_expr(expr);
        match src {
            ir::MatchSource::If => self.unify(expr.span, self.tcx.types.boolean, expr_ty),
            ir::MatchSource::Match => {}
        };

        // check that each arm pattern is the same type as the scrutinee
        for arm in arms {
            self.check_pat(arm.pat, expr_ty);
        }

        // special case when match has no arms
        if arms.is_empty() {
            return self.tcx.types.unit;
        }

        // otherwise, consider the last arm's body to be the expected type
        let n = arms.len() - 1;
        let expected_ty = self.check_expr(arms[n].body);
        arms[..n].iter().for_each(|arm| {
            let arm_ty = self.check_expr(arm.body);
            arm.guard.iter().for_each(|guard| {
                let guard_ty = self.check_expr(guard);
                self.unify(guard.span, self.tcx.types.boolean, guard_ty);
            });
            self.unify(arm.span, expected_ty, arm_ty);
        });
        expected_ty
    }

    fn check_call_expr(&mut self, expr: &ir::Expr, f: &ir::Expr, args: &[ir::Expr]) -> Ty<'tcx> {
        let ret_ty = self.new_infer_var(expr.span);
        let f_ty = self.check_expr(f);
        let arg_tys = self.check_expr_list(args);
        let ty = self.tcx.mk_ty(TyKind::Fn(arg_tys, ret_ty));
        self.unify(expr.span, f_ty, ty);
        ret_ty
    }

    fn check_closure_expr(
        &mut self,
        closure: &ir::Expr,
        sig: &ir::FnSig,
        body: &ir::Body,
    ) -> Ty<'tcx> {
        // the resolver resolved the closure name to the id of the entire closure expr
        // so we define an immutable local variable for it with the closure's type
        let clsr_ty = self.fn_sig_to_ty(sig);
        // self.record_upvars(closure, body);
        self.def_local(closure.id, clsr_ty, Mutability::Imm);
        let _fcx = self.check_fn(clsr_ty, body);
        clsr_ty
    }

    /// inputs are the types from the type signature (or inference variables)
    /// adds the parameters to locals and typechecks the expr of the body
    pub fn check_body(&mut self, body: &ir::Body) {
        for (param, ty) in body.params.iter().zip(self.param_tys) {
            self.check_pat(param.pat, ty);
        }
        let body_ty = self.check_expr(body.expr);
        self.unify(body.expr.span, self.ret_ty, body_ty);
        // explicitly overwrite the type of body with the return type of the function
        // in the case where it is inferred to be `!`
        // this is a special case due to return statements in the top level block expr
        // without this overwrite, if the final statement is diverging (i.e. return)
        // then the body function will be recorded to have type ! which is not correct
        self.write_ty(body.id(), self.ret_ty);
    }

    fn check_expr_list(&mut self, xs: &[ir::Expr]) -> SubstsRef<'tcx> {
        let tcx = self.tcx;
        let tys = xs.iter().map(|expr| self.check_expr(expr));
        tcx.mk_substs(tys)
    }

    fn check_expr_tuple(&mut self, xs: &[ir::Expr]) -> Ty<'tcx> {
        let tcx = self.tcx;
        let tys = xs.iter().map(|expr| self.check_expr(expr));
        tcx.mk_tup_iter(tys)
    }

    crate fn check_expr_path(&mut self, path: &ir::Path) -> Ty<'tcx> {
        match path.res {
            ir::Res::Local(id) => self.local_ty(id).ty,
            ir::Res::Def(def_id, def_kind) => self.check_expr_path_def(path.span, def_id, def_kind),
            ir::Res::PrimTy(_) => panic!("found type resolution in value namespace"),
            ir::Res::Err => self.set_ty_err(),
            ir::Res::SelfTy => todo!(),
        }
    }

    fn check_expr_path_def(&mut self, span: Span, def_id: DefId, def_kind: DefKind) -> Ty<'tcx> {
        match def_kind {
            // instantiate ty params
            DefKind::Fn | DefKind::Enum | DefKind::Struct | DefKind::Ctor(..) =>
                self.instantiate(span, self.collected_ty(def_id)),
            DefKind::AssocFn => todo!(),
            DefKind::Extern | DefKind::TyParam(_) | DefKind::Impl => unreachable!(),
        }
    }

    fn check_block(&mut self, block: &ir::Block) -> Ty<'tcx> {
        block.stmts.iter().for_each(|stmt| self.check_stmt(stmt));
        match &block.expr {
            Some(expr) => self.check_expr(expr),
            None => self.tcx.types.unit,
        }
    }

    fn check_binop(&mut self, op: ast::BinOp, l: &ir::Expr, r: &ir::Expr) -> Ty<'tcx> {
        let tl = self.check_expr(l);
        let tr = self.check_expr(r);
        match op {
            // TODO deal with floats
            BinOp::Mul | BinOp::Div | BinOp::Add | BinOp::Sub => {
                self.unify(l.span, self.tcx.types.int, tl);
                self.unify(r.span, tl, tr);
                tl
            }
            BinOp::Lt | BinOp::Gt => {
                // TODO deal with floats
                self.unify(l.span, self.tcx.types.int, tl);
                self.unify(r.span, tl, tr);
                self.tcx.types.boolean
            }
            BinOp::Eq => todo!(),
            BinOp::Neq => todo!(),
            BinOp::And | BinOp::Or => {
                self.unify(l.span, self.tcx.types.int, tl);
                self.unify(r.span, tl, tr);
                tl
            }
        }
    }

    fn check_lit(&self, lit: &ast::Lit) -> Ty<'tcx> {
        match lit {
            Lit::Float(_) => self.tcx.types.float,
            Lit::Bool(_) => self.tcx.types.boolean,
            Lit::Int(_) => self.tcx.types.int,
        }
    }
}
