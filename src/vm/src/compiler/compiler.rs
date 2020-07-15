use crate::ast;
use crate::exec::{CodeBuilder, Function, Op};
use crate::ir::{self, LocalId};
use crate::tir;
use crate::ty::TyKind;
use crate::typeck::TyCtx;
use rustc_hash::FxHashMap;
use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub(super) struct Compiler {
    code: CodeBuilder,
    /// mapping of local_var_id -> stack slot
    locals: FxHashMap<LocalId, u8>,
    /// use a u8 as locals can only be indexed using a single byte
    localc: u8,
}

impl Compiler {
    pub fn new() -> Self {
        Self { code: Default::default(), locals: Default::default(), localc: 0 }
    }

    pub fn finish(&mut self) -> Function {
        self.code.emit_op(Op::ret);
        Function::new(self.code.build())
    }

    pub fn compile_expr(&mut self, expr: &tir::Expr) {
        match expr.kind {
            tir::ExprKind::Lit(lit) => self.compile_expr_lit(lit),
            tir::ExprKind::Bin(op, l, r) => self.compile_expr_binary(op, l, r),
            tir::ExprKind::Unary(op, expr) => self.compile_expr_unary(op, expr),
            tir::ExprKind::Block(block) => self.compile_block(block),
            tir::ExprKind::VarRef(id) => self.compile_var_ref(id),
            tir::ExprKind::Tuple(xs) => self.compile_tuple(xs),
        };
    }

    fn compile_tuple(&mut self, xs: &[tir::Expr]) {
        xs.iter().for_each(|x| self.compile_expr(x));
        self.emit_tuple(xs.len() as u8);
    }

    fn compile_var_ref(&mut self, id: ir::Id) {
        let local_idx = *self.locals.get(&id.local).unwrap();
        self.emit_loadl(local_idx);
    }

    fn compile_block(&mut self, block: &tir::Block) {
        block.stmts.iter().for_each(|stmt| self.compile_stmt(stmt));
        block.expr.map(|expr| self.compile_expr(expr));
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

    fn compile_let_stmt(&mut self, l: &tir::Let) {
        // if no initializer, just put a `unit` in the slot
        match l.init {
            Some(expr) => self.compile_expr(expr),
            None => self.unit(),
        };
        match l.pat.kind {
            /// if its a wildcard, we don't bind anything so just pop the expression off
            tir::PatternKind::Wildcard => return self.pop(),
            tir::PatternKind::Binding(ident, _) => {
                // this relies on the observation that the nth local variable resides
                // in slot n of the current frame
                self.locals.insert(l.pat.id.local, self.localc);
                self.localc += 1;
            }
            tir::PatternKind::Field(fields) => match l.pat.ty.kind {
                TyKind::Tuple(_) => todo!(),
                _ => todo!(),
            },
        }
    }

    fn compile_expr_lit(&mut self, lit: ast::Lit) {
        match lit {
            ast::Lit::Num(d) => self.emit_dconst(d),
            ast::Lit::Bool(b) => self.emit_uconst(b as u64),
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
        self.emit_op(Op::pop);
    }
}

impl Deref for Compiler {
    type Target = CodeBuilder;

    fn deref(&self) -> &Self::Target {
        &self.code
    }
}

impl DerefMut for Compiler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.code
    }
}
