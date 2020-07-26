use super::Compiler;
use crate::ast;
use crate::exec::Op;
use crate::ir::{self, DefId};
use crate::tir;
use crate::ty::{Const, ConstKind, Ty, TyKind};
use indexed_vec::Idx;

impl<'tcx> Compiler<'tcx> {
    pub(super) fn compile_expr(&mut self, expr: &tir::Expr) {
        match expr.kind {
            tir::ExprKind::Lit(c) => self.compile_expr_lit(c, expr.ty),
            tir::ExprKind::Bin(op, l, r) => self.compile_expr_binary(op, l, r),
            tir::ExprKind::Unary(op, expr) => self.compile_expr_unary(op, expr),
            tir::ExprKind::Block(block) => self.compile_block(block),
            tir::ExprKind::VarRef(id) => self.compile_var_ref(id),
            tir::ExprKind::ItemRef(def_id) => self.compile_item_ref(def_id),
            tir::ExprKind::Tuple(xs) => self.compile_tuple(xs),
            tir::ExprKind::Lambda(f) => self.compile_lambda(f),
            tir::ExprKind::Call(f, args) => self.compile_call(f, args),
            tir::ExprKind::Match(expr, arms) => self.compile_match(expr, arms),
        };
    }

    fn compile_block(&mut self, block: &tir::Block) {
        self.with_scope(|compiler| {
            block.stmts.iter().for_each(|stmt| compiler.compile_stmt(stmt));
            // blocks are expressions and must always produce a value
            match block.expr {
                Some(expr) => compiler.compile_expr(expr),
                None => compiler.unit(),
            }
        })
    }

    fn compile_var_ref(&mut self, id: ir::Id) {
        let local_idx = self.find_local_slot(id.local);
        self.emit_loadl(local_idx);
    }

    fn compile_item_ref(&mut self, id: DefId) {
        let const_id = self.ctx.def_id_to_const_id.borrow()[&id];
        self.emit_ldc(const_id.index() as u8);
    }

    fn compile_lambda(&mut self, f: &tir::Body) {
        todo!()
    }

    fn compile_call(&mut self, f: &tir::Expr, args: &[tir::Expr]) {
        self.compile_expr(f);
        args.iter().for_each(|arg| self.compile_expr(arg));
        self.emit_invoke(args.len() as u8);
    }

    fn compile_tuple(&mut self, xs: &[tir::Expr]) {
        xs.iter().for_each(|x| self.compile_expr(x));
        self.emit_tuple(xs.len() as u8);
    }

    fn compile_match(&mut self, expr: &tir::Expr, arms: &[tir::Arm]) {
        self.compile_expr(expr);
        let mut arm_ends = vec![];
        for arm in arms {
            self.emit_op(Op::dup);
            self.compile_arm_pat(arm.pat);
            self.with_jmp(Op::jmpneq, |compiler| {
                compiler.compile_expr(arm.body);
                arm_ends.push(compiler.emit_jmp(Op::jmp));
            });
        }
        let match_end = self.code_len();
        for i in arm_ends {
            let offset = match_end - i;
            self.patch_jmp(i, offset as u16);
        }
    }

    fn compile_expr_lit(&mut self, c: &Const, ty: Ty) {
        match (c.kind, &ty.kind) {
            (ConstKind::Integral(i), TyKind::Bool) => self.emit_uconst(i),
            (ConstKind::Integral(c), TyKind::Char) => self.emit_uconst(c),
            (ConstKind::Floating(f), TyKind::Num) => self.emit_dconst(f),
            _ => unreachable!("type error"),
        };
    }

    fn compile_expr_unary(&mut self, op: ast::UnaryOp, expr: &tir::Expr) {
        self.compile_expr(expr);
        // let opcode = match op {
        //     ast::UnaryOp::Neg => Op::dneg,
        //     ast::UnaryOp::Not => Op::
        // }
        todo!()
    }

    fn compile_expr_binary(&mut self, op: ast::BinOp, l: &tir::Expr, r: &tir::Expr) {
        self.compile_expr(l);
        self.compile_expr(r);
        let opcode = match op {
            ast::BinOp::Mul => Op::dmul,
            ast::BinOp::Div => Op::ddiv,
            ast::BinOp::Add => Op::dadd,
            ast::BinOp::Sub => Op::dsub,
            ast::BinOp::Lt => Op::dcmplt,
            ast::BinOp::Gt => Op::dcmpgt,
        };
        self.emit_op(opcode);
    }
}
