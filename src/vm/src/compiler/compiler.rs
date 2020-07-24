use super::{CompilerCtx, ConstantPool};
use crate::ast;
use crate::exec::{CodeBuilder, Function, Op};
use crate::ir::{self, DefId, LocalId};
use crate::tir;
use crate::ty::{Const, Ty, TyKind};
use crate::typeck::TyCtx;
use ast::Lit;
use indexed_vec::{Idx, IndexVec};
use rustc_hash::FxHashMap;
use std::ops::{Deref, DerefMut};

pub(super) struct Compiler<'tcx> {
    code: CodeBuilder,
    locals: Vec<LocalId>,
    ctx: &'tcx CompilerCtx<'tcx>,
}

impl<'tcx> Compiler<'tcx> {
    pub fn new(ctx: &'tcx CompilerCtx<'tcx>) -> Self {
        Self { ctx, code: Default::default(), locals: Default::default() }
    }

    pub fn finish(&mut self) -> Function {
        self.code.emit_op(Op::ret);
        Function::new(self.code.build())
    }

    pub fn compile_expr(&mut self, expr: &tir::Expr) {
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

    fn compile_match(&mut self, expr: &tir::Expr, arms: &[tir::Arm]) {
        self.compile_expr(expr);
        let mut arm_ends = vec![];
        for arm in arms {
            self.compile_arm(arm);
            arm_ends.push(self.len());
            self.emit_jmp(Op::jmp, u16::MAX);
        }
        let match_end = self.len() as u16;
        for i in arm_ends {
            self.patch_jmp(i, match_end);
        }
    }

    fn compile_arm(&mut self, arm: &tir::Arm) {
        self.compile_pat(arm.pat);
        self.emit_op(Op::dup);
        self.with_jmp(Op::jmpeq, |compiler| {
            compiler.compile_expr(arm.body);
        });
    }

    /// emits a `jmp` instruction that will jump over the code generated by `f`
    fn with_jmp(&mut self, jmp_op: Op, f: impl FnOnce(&mut Self)) {
        let jmp_start = self.len();
        self.emit_jmp(jmp_op, u16::MAX);
        f(self);
        // -2 for the two `offset` bytes
        let offset = (self.len() - jmp_start - 2) as u16;
        self.patch_jmp(jmp_start, offset);
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

    fn compile_item_ref(&mut self, id: DefId) {
        let const_id = self.ctx.def_id_to_const_id.borrow()[&id];
        self.emit_ldc(const_id.index() as u8);
    }

    /// returns the `local_idx` for a variable with given `local_id`
    fn find_local_slot(&mut self, local_id: LocalId) -> u8 {
        self.locals.iter().rposition(|&id| id == local_id).unwrap() as u8
    }

    fn compile_var_ref(&mut self, id: ir::Id) {
        let local_idx = self.find_local_slot(id.local);
        self.emit_loadl(local_idx);
    }

    fn with_scope<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let m = self.locals.len();
        let ret = f(self);
        let n = self.locals.len();
        assert!(n >= m);
        // `n - m` locals were declared in the scope of the new block
        // they need to be popped off at the end of the scope
        // we also need to take care to retain the value of the block (as blocks are exprs)
        // this is done with the novel `popscp` (pop_scope) instruction
        let p = n - m;
        // leave only the first `m` locals as the rest are now out of scope
        self.locals.truncate(m);
        // redundant if p == 0
        if p > 0 {
            self.emit_popscp(p as u8);
        }
        ret
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

    fn compile_stmt(&mut self, stmt: &tir::Stmt) {
        match stmt.kind {
            tir::StmtKind::Let(l) => self.compile_let_stmt(l),
            tir::StmtKind::Expr(expr) => {
                self.compile_expr(expr);
                self.pop();
            }
        }
    }

    fn compile_pat(&mut self, pat: &tir::Pattern) {
        match pat.kind {
            tir::PatternKind::Lit(c) => self.compile_expr_lit(c, pat.ty),
            // if its a wildcard, we don't bind anything so just pop the expression off
            tir::PatternKind::Wildcard => return self.pop(),
            tir::PatternKind::Binding(ident, _) => {
                // this relies on the observation that the `n`th local variable resides
                // in slot `n` of the current frame
                self.locals.push(pat.id.local);
            }
            tir::PatternKind::Field(fields) => match pat.ty.kind {
                TyKind::Tuple(tys) => {
                    // need to unpack the tuple
                    fields.iter().for_each(|field| self.compile_pat(field.pat));
                    todo!()
                }
                _ => todo!(),
            },
        }
    }

    fn compile_let_stmt(&mut self, l: &tir::Let) {
        // if no initializer, just put a `unit` in the slot
        match l.init {
            Some(expr) => self.compile_expr(expr),
            None => self.unit(),
        };
        self.compile_pat(l.pat);
    }

    fn compile_expr_lit(&mut self, c: &Const, ty: Ty) {
        match ty.kind {
            TyKind::Bool => self.emit_uconst(c.val),
            TyKind::Char => self.emit_uconst(c.val),
            TyKind::Num => self.emit_dconst(f64::from_bits(c.val)),
            _ => unreachable!(),
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
        };
        self.emit_op(opcode);
    }

    /// emits pop instruction
    fn pop(&mut self) {
        self.emit_op(Op::pop);
    }

    /// emits unit instruction
    fn unit(&mut self) {
        self.emit_op(Op::unit);
    }
}

impl<'tcx> Deref for Compiler<'tcx> {
    type Target = CodeBuilder;

    fn deref(&self) -> &Self::Target {
        &self.code
    }
}

impl<'tcx> DerefMut for Compiler<'tcx> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.code
    }
}
