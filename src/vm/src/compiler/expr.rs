use super::{Compiler, Constant, FrameCtx};
use crate::ast;
use crate::exec::{Function, Op};
use crate::ir::{self, DefId};
use crate::tir;
use crate::ty::{Const, ConstKind, Ty, TyKind};
use indexed_vec::Idx;

impl<'tcx> Compiler<'tcx> {
    pub(super) fn compile_expr(&mut self, expr: &tir::Expr) {
        match expr.kind {
            tir::ExprKind::Const(c) => self.compile_expr_lit(c, expr.ty),
            tir::ExprKind::Bin(op, l, r) => self.compile_expr_binary(op, l, r),
            tir::ExprKind::Unary(op, expr) => self.compile_expr_unary(op, expr),
            tir::ExprKind::Block(block) => self.compile_block(block),
            tir::ExprKind::VarRef(id) => self.compile_var_ref(id),
            tir::ExprKind::ItemRef(def_id) => self.compile_item_ref(def_id),
            tir::ExprKind::Tuple(xs) => self.compile_tuple(xs),
            tir::ExprKind::Lambda(f) => self.compile_lambda(f),
            tir::ExprKind::Ret(expr) => self.compile_ret(expr),
            tir::ExprKind::Call(f, args) => self.compile_call(f, args),
            tir::ExprKind::Match(expr, arms) => self.compile_match(expr, arms),
            tir::ExprKind::Assign(l, r) => self.compile_assign(l, r),
        };
    }

    fn compile_ret(&mut self, expr: Option<&tir::Expr>) {
        match expr {
            Some(expr) => self.compile_expr(expr),
            None => self.unit(),
        }
        self.emit_op(Op::ret);
    }

    fn compile_assign(&mut self, l: &tir::Expr, r: &tir::Expr) {
        let var_id = match l.kind {
            tir::ExprKind::VarRef(id) => id,
            _ => unreachable!(),
        };
        self.compile_expr(r);
        if let Some(local_idx) = self.resolve_local(var_id) {
            self.emit_istorel(local_idx);
        } else {
            let upvar_idx = self.resolve_upvar(var_id);
            self.emit_istoreu(upvar_idx);
        }
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
        if let Some(local_idx) = self.resolve_local(id) {
            self.emit_loadl(local_idx);
        } else {
            let upvar_idx = self.resolve_upvar(id);
            self.emit_loadu(upvar_idx);
        }
    }

    /// gets the `scope`th from top `FrameCtx`
    fn frame_mut(&mut self, scope: usize) -> &mut FrameCtx<'tcx> {
        let n = self.frames.len();
        &mut self.frames[n - scope - 1]
    }

    fn resolve_upvar(&mut self, id: ir::Id) -> u8 {
        self.resolve_upvar_inner(id, 0)
    }

    fn resolve_upvar_inner(&mut self, id: ir::Id, scope: usize) -> u8 {
        let n = self.frames.len();
        if scope == n {
            panic!("unresolved upvar")
        }
        // start by searching the enclosing `FrameCtx` hence the `1+`
        match Self::resolve_frame_local(self.frame_mut(1 + scope), id) {
            Some(upvar_idx) => self.frame_mut(scope).mk_upvar(true, upvar_idx),
            None => {
                // if not resolved, recursively search outer `FrameCtx`
                let upvar_idx = self.resolve_upvar_inner(id, 1 + scope);
                self.frame_mut(scope).mk_upvar(false, upvar_idx)
            }
        }
    }

    fn compile_item_ref(&mut self, id: DefId) {
        let const_id = self.gctx.def_id_to_const_id.borrow()[&id];
        self.emit_ldc(const_id.index() as u8);
    }

    fn compile_lambda(&mut self, body: &tir::Body) {
        let (lambda_idx, upvars) = self.with_frame(|compiler| {
            compiler.compile_body(body);
            let code = compiler.finish();
            let lambda = Function::new(code);
            let upvars = std::mem::take(&mut compiler.upvars);
            (compiler.gctx.mk_const(Constant::Lambda(lambda)).index() as u8, upvars)
        });
        self.emit_closure(lambda_idx, upvars);
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
            (ConstKind::Bool(i), TyKind::Bool) => self.emit_uconst(i),
            (ConstKind::Bool(c), TyKind::Char) => self.emit_uconst(c),
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
