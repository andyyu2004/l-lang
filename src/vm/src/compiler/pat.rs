use super::Compiler;
use crate::ast;
use crate::exec::Op;
use crate::ir::{self, DefId};
use crate::tir;
use crate::ty::{Const, ConstKind, Ty, TyKind};
use indexed_vec::Idx;

impl<'tcx> Compiler<'tcx> {
    pub(super) fn compile_arm_pat(&mut self, pat: &tir::Pattern) {
        match pat.kind {
            tir::PatternKind::Lit(expr) => self.compile_expr(expr),
            // if its a wildcard, we don't bind anything so just pop the expression off
            tir::PatternKind::Wildcard => {
                // as this will always match, we can just duplicate the top of the stack such that
                // the body of this pattern will be run duplicate top as the body of a pattern
                // is run if the top two values of the stack are equal
                // this pattern would work for any irrefutable pattern
                self.emit_op(Op::dup);
            }
            tir::PatternKind::Binding(ident, _) => {
                // this relies on the observation that the `n`th local variable resides
                // in slot `n` of the current frame
                self.locals.push(pat.id.local);
                self.emit_op(Op::dup);
            }
            tir::PatternKind::Field(fields) => todo!(),
        };
    }

    // TODO need to consider refutability at some point
    pub(super) fn compile_let_pat(&mut self, pat: &tir::Pattern) {
        match pat.kind {
            // if its a wildcard, we don't bind anything so just pop the expression off
            tir::PatternKind::Wildcard => self.pop(),
            tir::PatternKind::Binding(ident, _) => {
                // this relies on the observation that the `n`th local variable resides
                // in slot `n` of the current frame
                self.locals.push(pat.id.local);
            }
            tir::PatternKind::Field(fields) => match pat.ty.kind {
                TyKind::Tuple(tys) => {
                    // need to unpack the tuple
                    fields.iter().for_each(|field| self.compile_let_pat(field.pat));
                    todo!()
                }
                _ => todo!(),
            },
            tir::PatternKind::Lit(_) => unreachable!("literal pattern is refutable"),
        }
    }
}
