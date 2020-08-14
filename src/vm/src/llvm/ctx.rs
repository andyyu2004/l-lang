use super::util::LLVMAsPtrVal;
use crate::ast;
use crate::ir::{self, DefId};
use crate::lexer::symbol;
use crate::tir;
use crate::ty::{Const, ConstKind, SubstsRef, Ty, TyKind};
use crate::typeck::TyCtx;
use inkwell::types::{BasicType, BasicTypeEnum, FloatType, FunctionType};
use inkwell::values::*;
use inkwell::{
    basic_block::BasicBlock, builder::Builder, context::Context, module::Module, passes::PassManager
};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate};
use itertools::Itertools;
use rustc_hash::{FxHashMap, FxHashSet};
use std::fmt::Display;

pub struct CodegenCtx<'tcx> {
    pub tcx: TyCtx<'tcx>,
    pub ctx: &'tcx Context,
    pub builder: Builder<'tcx>,
    pub fpm: PassManager<FunctionValue<'tcx>>,
    pub module: Module<'tcx>,
    pub vals: CommonValues<'tcx>,
    curr_fn: Option<FunctionValue<'tcx>>,
    vars: FxHashMap<ir::Id, PointerValue<'tcx>>,
    dead_blocks: FxHashSet<BasicBlock<'tcx>>,
}

pub struct CommonValues<'tcx> {
    zero: IntValue<'tcx>,
}

impl<'tcx> CodegenCtx<'tcx> {
    pub fn new(tcx: TyCtx<'tcx>, ctx: &'tcx Context) -> Self {
        let module = ctx.create_module("main");
        let fpm = PassManager::create(&module);
        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.add_gvn_pass();
        fpm.add_cfg_simplification_pass();
        fpm.add_basic_alias_analysis_pass();
        fpm.add_promote_memory_to_register_pass();
        fpm.add_instruction_combining_pass();
        fpm.add_reassociate_pass();
        fpm.initialize();
        let vals = CommonValues { zero: ctx.i64_type().const_zero() };
        Self {
            tcx,
            ctx,
            module,
            fpm,
            builder: ctx.create_builder(),
            vals,
            curr_fn: None,
            dead_blocks: Default::default(),
            vars: Default::default(),
        }
    }

    /// returns the main function
    pub fn compile(&mut self, prog: &tir::Prog) -> FunctionValue<'tcx> {
        let mut main = None;
        prog.items.values().for_each(|item| self.def_item(item));
        for item in prog.items.values() {
            let f = self.compile_item(item);
            if item.ident.symbol == symbol::MAIN {
                main = Some(f);
            }
        }
        self.module.verify().unwrap();
        self.module.print_to_file("ir.ll").unwrap();
        main.unwrap()
    }

    fn def_item(&mut self, item: &tir::Item) {
        match item.kind {
            tir::ItemKind::Fn(ty, _, body) => {
                // types
                let (_forall, fn_ty) = ty.expect_scheme();
                let (arg_tys, ret_ty) = fn_ty.expect_fn();
                let llvm_arg_tys = arg_tys.into_iter().map(|ty| self.llvm_ty(ty)).collect_vec();
                let llvm_fn_ty = self.llvm_ty(ret_ty).fn_type(&llvm_arg_tys, false);
                let fn_val = self.module.add_function(&item.id.def.to_string(), llvm_fn_ty, None);
                // define parameters
                for (param, arg) in body.params.into_iter().zip(fn_val.get_param_iter()) {
                    arg.set_name(&param.pat.id.to_string());
                }
            }
        }
    }

    fn compile_item(&mut self, item: &tir::Item) -> FunctionValue<'tcx> {
        match item.kind {
            tir::ItemKind::Fn(ty, generics, body) => self.compile_fn(item, ty, generics, body),
        }
    }

    fn with_fn<R>(&mut self, new_fn: FunctionValue<'tcx>, f: impl FnOnce(&mut Self) -> R) -> R {
        let prev = self.curr_fn.take();
        self.curr_fn = Some(new_fn);
        let ret = f(self);
        self.curr_fn = prev;
        ret
    }

    fn remove_dead_blocks(&mut self) {
        std::mem::take(&mut self.dead_blocks)
            .into_iter()
            .for_each(|b| b.remove_from_function().unwrap());
    }

    fn compile_fn(
        &mut self,
        item: &tir::Item,
        ty: Ty,
        generics: &tir::Generics,
        body: &tir::Body,
    ) -> FunctionValue<'tcx> {
        let fn_val = self.module.get_function(&item.id.def.to_string()).unwrap();
        self.with_fn(fn_val, |this| {
            this.compile_body(body);
            this.remove_dead_blocks();
            this.module.print_to_stderr();
            assert!(fn_val.verify(true));
            this.fpm.run_on(&fn_val);
        });
        fn_val
    }

    fn compile_body(&mut self, body: &tir::Body) {
        let fn_val = self.curr_fn.unwrap();
        let basic_block = self.ctx.append_basic_block(fn_val, "body");
        self.with_block(basic_block, |this| {
            // params
            for (param, arg) in body.params.iter().zip(fn_val.get_param_iter()) {
                let ptr = this.compile_let_pat(param.pat);
                this.builder.build_store(ptr, arg);
            }
            // body
            let body = this.compile_expr(body.expr);
            this.builder.build_return(Some(&body));
        });
    }

    fn compile_let_pat(&mut self, pat: &tir::Pattern) -> PointerValue<'tcx> {
        match pat.kind {
            tir::PatternKind::Wildcard => self.alloca(pat.ty, pat.id),
            tir::PatternKind::Binding(ident, _) => {
                let ptr = self.alloca(pat.ty, pat.id);
                self.def_var(pat.id, ptr);
                ptr
            }
            tir::PatternKind::Field(_) => todo!(),
            tir::PatternKind::Lit(_) => unreachable!("refutable"),
        }
    }

    fn llvm_fn_ty(&self, params: SubstsRef, ret: Ty) -> FunctionType<'tcx> {
        self.llvm_ty(ret)
            .fn_type(&params.into_iter().map(|ty| self.llvm_ty(ty)).collect_vec(), false)
    }

    fn llvm_ty(&self, ty: Ty) -> BasicTypeEnum<'tcx> {
        match ty.kind {
            TyKind::Bool => self.ctx.bool_type().into(),
            TyKind::Char => todo!(),
            TyKind::Num => self.ctx.f64_type().into(),
            TyKind::Array(ty) => todo!(),
            TyKind::Fn(params, ret) =>
                self.llvm_fn_ty(params, ret).ptr_type(AddressSpace::Generic).into(),
            TyKind::Tuple(_) => todo!(),
            TyKind::Param(_) => todo!(),
            TyKind::Scheme(_, _) => todo!(),
            TyKind::Never => self.mk_unit_ty(),
            TyKind::Error | TyKind::Infer(_) => unreachable!(),
            TyKind::Adt(..) => todo!(),
        }
    }

    fn compile_expr(&mut self, expr: &tir::Expr) -> BasicValueEnum<'tcx> {
        match expr.kind {
            tir::ExprKind::Const(c) => self.compile_const(c),
            tir::ExprKind::Bin(op, l, r) => self.compile_bin(op, l, r),
            tir::ExprKind::Unary(_, _) => todo!(),
            tir::ExprKind::Block(block) => self.compile_block(block),
            tir::ExprKind::VarRef(id) => self.compile_var_ref(id),
            tir::ExprKind::ItemRef(def_id) => self.compile_item_ref(def_id),
            tir::ExprKind::Tuple(_) => todo!(),
            tir::ExprKind::Lambda(body) => self.compile_lambda(expr, body).into(),
            tir::ExprKind::Call(f, args) => self.compile_call(f, args),
            tir::ExprKind::Match(scrut, arms) => self.compile_match(expr, scrut, arms),
        }
    }

    fn compile_lambda(&mut self, expr: &tir::Expr, body: &tir::Body) -> PointerValue<'tcx> {
        let (param_tys, ret_ty) = expr.ty.expect_fn();
        let fn_val = self.module.add_function(
            &expr.id.to_string(),
            self.llvm_fn_ty(param_tys, ret_ty),
            None,
        );
        self.with_fn(fn_val, |this| this.compile_body(body));
        fn_val.as_llvm_ptr()
    }

    fn compile_arm_pat(&mut self, pat: &tir::Pattern, cmp_val: IntValue<'tcx>) -> IntValue<'tcx> {
        match pat.kind {
            tir::PatternKind::Wildcard => cmp_val,
            tir::PatternKind::Binding(ident, subpat) => {
                self.compile_let_pat(pat);
                cmp_val
            }
            tir::PatternKind::Lit(expr) => self.compile_expr(expr).into_int_value(),
            tir::PatternKind::Field(_) => todo!(),
        }
    }

    /// restores the builder to the end of the block after executing f
    fn with_builder<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let prev_block = self.builder.get_insert_block().unwrap();
        let ret = f(self);
        self.builder.position_at_end(prev_block);
        ret
    }

    /// writes to the given block, and restores the builder to the end of the previous block
    fn with_block<R>(&mut self, block: BasicBlock<'tcx>, f: impl FnOnce(&mut Self) -> R) -> R {
        let prev_block = self.builder.get_insert_block();
        self.builder.position_at_end(block);
        let ret = f(self);
        self.builder.position_at_end(prev_block.unwrap_or(block));
        ret
    }

    fn build_unreachable_block(&mut self) -> BasicBlock<'tcx> {
        let bb = self.ctx.append_basic_block(self.curr_fn(), "unreachable_block");
        self.with_block(bb, |this| this.builder.build_unreachable());
        bb
    }

    fn compile_match(
        &mut self,
        expr: &tir::Expr,
        scrut: &tir::Expr,
        arms: &[tir::Arm],
    ) -> BasicValueEnum<'tcx> {
        let cmp_val = self.compile_expr(scrut).into_int_value();
        let mut cmp_blocks = Vec::with_capacity(arms.len());
        let mut body_blocks = Vec::with_capacity(1 + arms.len());
        let mut arm_vals = Vec::with_capacity(arms.len());
        arms.iter().enumerate().for_each(|(i, arm)| {
            cmp_blocks.push(self.ctx.append_basic_block(self.curr_fn(), &format!("arm_cmp_{}", i)));
            body_blocks.push(self.ctx.append_basic_block(self.curr_fn(), &format!("arm_{}", i)));
        });
        self.builder.build_unconditional_branch(cmp_blocks[0]);
        let default_block = self.build_unreachable_block();
        let match_end_block = self.ctx.append_basic_block(self.curr_fn(), "match_end");
        arms.iter().enumerate().for_each(|(i, arm)| {
            // build arm cmp
            self.with_block(cmp_blocks[i], |this| {
                let cond_val = this.compile_arm_pat(arm.pat, cmp_val);
                let cond = this.builder.build_int_compare(
                    IntPredicate::EQ,
                    cmp_val,
                    cond_val,
                    "arm_cmp_val",
                );
                this.builder.build_conditional_branch(
                    cond,
                    body_blocks[i],
                    *cmp_blocks.get(i + 1).unwrap_or(&default_block),
                );
            });

            // build arm body
            self.with_block(body_blocks[i], |this| {
                arm_vals.push(this.compile_expr(arm.body));
                this.builder.build_unconditional_branch(match_end_block);
            });
        });
        // build merge block
        self.builder.position_at_end(match_end_block);
        let phi = self.builder.build_phi(self.llvm_ty(expr.ty), "match_phi");
        let incoming = arm_vals.iter().map(|v| v as &dyn BasicValue).zip(body_blocks).collect_vec();
        phi.add_incoming(incoming.as_slice());
        phi.as_basic_value()
    }

    fn compile_call(&mut self, f: &tir::Expr, args: &[tir::Expr]) -> BasicValueEnum<'tcx> {
        let f = self.compile_expr(f).into_pointer_value();
        let args = args.iter().map(|arg| self.compile_expr(arg)).collect_vec();
        self.builder.build_call(f, &args, "fcall").try_as_basic_value().left().unwrap()
    }

    fn compile_item_ref(&mut self, id: DefId) -> BasicValueEnum<'tcx> {
        self.module.get_function(&id.to_string()).unwrap().as_llvm_ptr().into()
    }

    fn compile_var_ref(&mut self, id: ir::Id) -> BasicValueEnum<'tcx> {
        let ptr = self.vars[&id];
        self.builder.build_load(ptr, "load")
    }

    fn def_var(&mut self, id: ir::Id, ptr: PointerValue<'tcx>) {
        self.vars.insert(id, ptr);
    }

    fn compile_stmt(&mut self, stmt: &tir::Stmt) {
        match stmt.kind {
            tir::StmtKind::Let(l) => {
                let v = l
                    .init
                    .map(|expr| self.compile_expr(expr))
                    .unwrap_or_else(|| self.vals.zero.into());
                let ptr = self.alloca(l.pat.ty, l.pat.id);
                self.def_var(l.pat.id, ptr);
                self.builder.build_store(ptr, v);
            }
            tir::StmtKind::Ret(ret_expr) => self.compile_ret(ret_expr),
            tir::StmtKind::Expr(expr) => {
                self.compile_expr(expr);
            }
        };
    }

    fn build_dead_block(&mut self) -> BasicBlock<'tcx> {
        let dead = self.ctx.append_basic_block(self.curr_fn(), "dead");
        self.builder.position_at_end(dead);
        self.dead_blocks.insert(dead);
        dead
    }

    fn compile_ret(&mut self, ret_expr: Option<&tir::Expr>) {
        // uhh ... must be a better to get a trait object from within the option
        let ret_val = ret_expr.map(|expr| box self.compile_expr(expr) as Box<dyn BasicValue>);
        self.builder.build_return(ret_val.as_deref());
        // create a dead block to write unreachable instructions in
        self.build_dead_block();
    }

    fn curr_fn(&self) -> FunctionValue<'tcx> {
        self.curr_fn.unwrap()
    }

    fn alloca(&mut self, ty: Ty, t: impl Display) -> PointerValue<'tcx> {
        self.with_builder(|this| {
            let basic_block = this.curr_fn().get_first_basic_block().unwrap();
            match &basic_block.get_first_instruction() {
                Some(inst) => this.builder.position_before(inst),
                None => this.builder.position_at_end(basic_block),
            };
            this.builder.build_alloca(this.llvm_ty(ty), &t.to_string())
        })
    }

    fn mk_unit(&self) -> BasicValueEnum<'tcx> {
        self.ctx.const_struct(&[], true).into()
    }

    fn mk_unit_ty(&self) -> BasicTypeEnum<'tcx> {
        self.mk_unit().get_type()
    }

    fn compile_block(&mut self, block: &tir::Block) -> BasicValueEnum<'tcx> {
        block.stmts.iter().for_each(|stmt| self.compile_stmt(stmt));
        block.expr.map(|expr| self.compile_expr(expr)).unwrap_or_else(|| self.mk_unit())
    }

    fn compile_bin(
        &mut self,
        op: ast::BinOp,
        l: &tir::Expr,
        r: &tir::Expr,
    ) -> BasicValueEnum<'tcx> {
        let lhs = self.compile_expr(l).into_float_value();
        let rhs = self.compile_expr(r).into_float_value();
        match op {
            ast::BinOp::Mul => self.builder.build_float_mul(lhs, rhs, "tmpfmul").into(),
            ast::BinOp::Div => self.builder.build_float_div(lhs, rhs, "tmpfdiv").into(),
            ast::BinOp::Add => self.builder.build_float_add(lhs, rhs, "tmpadd").into(),
            ast::BinOp::Sub => self.builder.build_float_sub(lhs, rhs, "tmpfsub").into(),
            ast::BinOp::Lt | ast::BinOp::Gt => self.compile_cmp(op, lhs, rhs).into(),
        }
    }

    fn compile_cmp(
        &mut self,
        op: ast::BinOp,
        l: FloatValue<'tcx>,
        r: FloatValue<'tcx>,
    ) -> IntValue<'tcx> {
        match op {
            ast::BinOp::Lt =>
                self.builder.build_float_compare(FloatPredicate::OLT, l, r, "fcmp_lt"),
            ast::BinOp::Gt =>
                self.builder.build_float_compare(FloatPredicate::OGT, l, r, "fcmp_gt"),
            ast::BinOp::Mul | ast::BinOp::Div | ast::BinOp::Add | ast::BinOp::Sub => unreachable!(),
        }
    }

    fn compile_const(&mut self, c: &Const) -> BasicValueEnum<'tcx> {
        match c.kind {
            ConstKind::Floating(f) => self.ctx.f64_type().const_float(f).into(),
            ConstKind::Bool(b) => self.ctx.bool_type().const_int(b, true).into(),
        }
    }
}
