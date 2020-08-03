use crate::ast;
use crate::ir::{self, DefId};
use crate::lexer::symbol;
use crate::tir;
use crate::ty::{Const, ConstKind, Ty, TyKind};
use crate::typeck::TyCtx;
use inkwell::types::{BasicType, BasicTypeEnum, FloatType};
use inkwell::values::*;
use inkwell::FloatPredicate;
use inkwell::{
    basic_block::BasicBlock, builder::Builder, context::Context, module::Module, passes::PassManager, IntPredicate
};
use itertools::Itertools;
use rustc_hash::FxHashMap;
use std::fmt::Display;

crate struct CodegenCtx<'tcx> {
    pub tcx: TyCtx<'tcx>,
    pub ctx: &'tcx Context,
    pub builder: Builder<'tcx>,
    pub fpm: PassManager<FunctionValue<'tcx>>,
    pub module: Module<'tcx>,
    pub vals: CommonValues<'tcx>,
    fns: Vec<FunctionValue<'tcx>>,
    vars: FxHashMap<ir::Id, PointerValue<'tcx>>,
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
            fns: Default::default(),
            vars: Default::default(),
        }
    }

    /// returns the main function
    pub fn compile(&mut self, prog: &tir::Prog) -> FunctionValue<'tcx> {
        let mut main = None;
        for item in prog.items.values() {
            let f = self.compile_item(item);
            if item.ident.symbol == symbol::MAIN {
                main = Some(f)
            }
        }
        main.unwrap()
    }

    fn compile_item(&mut self, item: &tir::Item) -> FunctionValue<'tcx> {
        match item.kind {
            tir::ItemKind::Fn(ty, generics, body) => self.compile_fn(item, ty, generics, body),
        }
    }

    fn compile_fn(
        &mut self,
        item: &tir::Item,
        ty: Ty,
        generics: &tir::Generics,
        body: &tir::Body,
    ) -> FunctionValue<'tcx> {
        // types
        let (_forall, fn_ty) = ty.expect_scheme();
        let (arg_tys, ret_ty) = fn_ty.expect_fn();
        let llvm_arg_tys = arg_tys.into_iter().map(|ty| self.llvm_ty(ty)).collect_vec();
        let llvm_fn_ty = self.llvm_ty(ret_ty).fn_type(&llvm_arg_tys, false);
        let fn_val = self.module.add_function(&item.id.def.to_string(), llvm_fn_ty, None);
        self.fns.push(fn_val);
        // params
        let basic_block = self.ctx.append_basic_block(fn_val, "fn_block");
        body.params.into_iter().for_each(|p| self.compile_let_pat(p.pat));
        // body
        let body = self.compile_expr(body.expr);
        // self.builder.position_at_end(basic_block);
        self.builder.build_return(Some(&body));
        self.module.print_to_stderr();
        assert!(fn_val.verify(true));
        self.fpm.run_on(&fn_val);
        fn_val
    }

    fn compile_let_pat(&mut self, pat: &tir::Pattern) {
        match pat.kind {
            tir::PatternKind::Wildcard => {}
            tir::PatternKind::Binding(ident, _) => {
                let ptr = self.alloca(pat.ty, pat.id);
                self.def(pat.id, ptr);
            }
            tir::PatternKind::Field(_) => todo!(),
            tir::PatternKind::Lit(_) => unreachable!("refutable"),
        }
    }

    fn llvm_ty(&self, ty: Ty) -> BasicTypeEnum<'tcx> {
        match ty.kind {
            TyKind::Bool => todo!(),
            TyKind::Char => todo!(),
            TyKind::Num => self.ctx.f64_type().into(),
            TyKind::Array(_) => todo!(),
            TyKind::Fn(_, _) => todo!(),
            TyKind::Tuple(_) => todo!(),
            TyKind::Param(_) => todo!(),
            TyKind::Scheme(_, _) => todo!(),
            TyKind::Error | TyKind::Infer(_) => unreachable!(),
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
            tir::ExprKind::Lambda(_) => todo!(),
            tir::ExprKind::Call(f, args) => self.compile_call(f, args),
            tir::ExprKind::Match(scrut, arms) => self.compile_match(expr, scrut, arms),
        }
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

    // fn compile_match(
    //     &mut self,
    //     expr: &tir::Expr,
    //     scrut: &tir::Expr,
    //     arms: &[tir::Arm],
    // ) -> BasicValueEnum<'tcx> {
    //     let original_block = self.curr_fn().get_last_basic_block().unwrap();
    //     let scrut_val = self.compile_expr(scrut).into_int_value();
    //     let match_end_block = self.ctx.append_basic_block(self.curr_fn(), "match_end_block");
    //     self.builder.position_at_end(match_end_block);
    //     self.builder.build_unreachable();
    //     let mut cmp_blocks = Vec::with_capacity(arms.len());
    //     let mut arm_blocks = Vec::with_capacity(arms.len());
    //     arms.into_iter().enumerate().for_each(|(i, arm)| {
    //         cmp_blocks
    //             .push(self.ctx.append_basic_block(self.curr_fn(), &format!("cmp_block_{}", i)));
    //         arm_blocks
    //             .push(self.ctx.append_basic_block(self.curr_fn(), &format!("arm_block_{}", i)));
    //     });

    //     let cases = arms
    //         .into_iter()
    //         .enumerate()
    //         .map(|(i, arm)| {
    //             self.builder.position_at_end(cmp_blocks[i]);
    //             let arm_cmp_val = self.compile_arm_pat(arm.pat, scrut_val);
    //             let cmp = self.builder.build_int_compare(
    //                 IntPredicate::EQ,
    //                 scrut_val,
    //                 arm_cmp_val,
    //                 "arm_cmp",
    //             );
    //             self.builder.build_conditional_branch(
    //                 cmp,
    //                 arm_blocks[i],
    //                 *cmp_blocks.get(i + 1).unwrap_or(&match_end_block),
    //             );

    //             self.builder.position_at_end(arm_blocks[i]);
    //             // this expr doesn't do anything, it needs to be "returned" somehow
    //             self.compile_expr(arm.body);
    //             self.builder.build_unconditional_branch(match_end_block);
    //             (arm_cmp_val, cmp_blocks[i])
    //         })
    //         .collect_vec();
    //     self.builder.position_at_end(original_block);
    //     // weird return value, check the comments on `build_switch`
    //     let value = self.builder.build_switch(scrut_val, match_end_block, &cases);
    //     let int_val: FloatValue = unsafe { std::mem::transmute(value) };
    //     dbg!(int_val);
    //     // TODO its probably better to use the if else stuff below with phi
    //     // build select also looks promising
    //     int_val.into()
    // }

    fn with_builder<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        todo!()
    }

    fn build_unreachable(&self) -> BasicBlock<'tcx> {
        let bb = self.ctx.append_basic_block(self.curr_fn(), "unreachable_block");
        self.builder.position_at_end(bb);
        self.builder.build_unreachable();
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
        arms.into_iter().enumerate().for_each(|(i, arm)| {
            cmp_blocks.push(self.ctx.append_basic_block(self.curr_fn(), &format!("arm_cmp_{}", i)));
            body_blocks.push(self.ctx.append_basic_block(self.curr_fn(), &format!("arm_{}", i)));
        });
        self.builder.build_unconditional_branch(cmp_blocks[0]);
        let default_block = self.build_unreachable();
        let match_end_block = self.ctx.append_basic_block(self.curr_fn(), "match_end");
        arms.into_iter().enumerate().for_each(|(i, arm)| {
            // build arm cmp
            self.builder.position_at_end(cmp_blocks[i]);
            let cond_val = self.compile_arm_pat(arm.pat, cmp_val);
            let cond =
                self.builder.build_int_compare(IntPredicate::EQ, cmp_val, cond_val, "arm_cmp_val");
            self.builder.build_conditional_branch(
                cond,
                body_blocks[i],
                *cmp_blocks.get(i + 1).unwrap_or(&default_block),
            );
            // build arm body
            self.builder.position_at_end(body_blocks[i]);
            arm_vals.push(self.compile_expr(arm.body));
            self.builder.build_unconditional_branch(match_end_block);
        });
        // build merge block
        self.builder.position_at_end(match_end_block);
        let phi = self.builder.build_phi(self.llvm_ty(expr.ty), "match_phi");
        let incoming = arm_vals.iter().map(|v| v as &dyn BasicValue).zip(body_blocks).collect_vec();
        phi.add_incoming(incoming.as_slice());
        phi.as_basic_value()
    }

    // fn compile_match(
    //     &mut self,
    //     expr: &tir::Expr,
    //     scrut: &tir::Expr,
    //     arms: &[tir::Arm],
    // ) -> BasicValueEnum<'tcx> {
    //     let val = self.compile_expr(scrut).into_int_value();
    //     let mut cmp_blocks = Vec::with_capacity(arms.len());
    //     // let mut body_blocks = Vec::with_capacity(arms.len());
    //     let mut arm_vals = vec![self.vals.zero.into(); arms.len()];

    //     let match_default_block = self.ctx.append_basic_block(self.curr_fn(), "match_default");
    //     self.builder.position_at_end(match_default_block);
    //     self.builder.build_unreachable();
    //     self.builder.build_unconditional_branch(cmp_blocks[0]);

    //     arms.into_iter().enumerate().for_each(|(i, arm)| {
    //         cmp_blocks.push(self.ctx.append_basic_block(self.curr_fn(), &format!("arm_cmp_{}", i)))
    //     });
    //     let match_end_block = self.ctx.append_basic_block(self.curr_fn(), "match_end");
    //     for (i, arm) in arms.into_iter().enumerate() {
    //         self.builder.position_at_end(cmp_blocks[i]);
    //         let arm_cmp_val = self.compile_arm_pat(arm.pat, val);
    //         let cmp = self.builder.build_int_compare(IntPredicate::EQ, val, arm_cmp_val, "arm_cmp");
    //         arm_vals[i] = self.compile_expr(arm.body);
    //         self.builder.get_insert_block()
    //         self.builder.build_conditional_branch(
    //             cmp,
    //             body_blocks[i],
    //             *cmp_blocks.get(i + 1).unwrap_or(&match_end_block),
    //         );
    //         self.builder.build_unconditional_branch(match_end_block);
    //     }
    //     self.builder.position_at_end(match_end_block);
    //     let phi = self.builder.build_phi(self.llvm_ty(expr.ty), "match_phi");
    //     debug_assert_eq!(cmp_blocks.len(), arm_vals.len());
    //     let incoming =
    //         arm_vals.iter().map(|val| val as &dyn BasicValue).zip(cmp_blocks).collect_vec();
    //     phi.add_incoming(incoming.as_slice());
    //     self.builder.position_at_end(match_end_block);
    //     phi.as_basic_value()
    // }

    fn compile_call(&mut self, f: &tir::Expr, args: &[tir::Expr]) -> BasicValueEnum<'tcx> {
        let f = self.compile_expr(f).into_pointer_value();
        let args = args.into_iter().map(|arg| self.compile_expr(arg)).collect_vec();
        self.builder.build_call(f, &args, "invoke").try_as_basic_value().left().unwrap()
    }

    fn compile_item_ref(&mut self, id: DefId) -> BasicValueEnum<'tcx> {
        let f = self.module.get_function(&id.to_string()).unwrap();
        // :) why is it so hard to get a pointer to a function
        let ptr: PointerValue<'tcx> = unsafe { std::mem::transmute(f) };
        ptr.into()
    }

    fn compile_var_ref(&mut self, id: ir::Id) -> BasicValueEnum<'tcx> {
        let ptr = self.vars[&id];
        self.builder.build_load(ptr, "load")
    }

    fn def(&mut self, id: ir::Id, ptr: PointerValue<'tcx>) {
        self.vars.insert(id, ptr);
    }

    fn compile_stmt(&mut self, stmt: &tir::Stmt) {
        match stmt.kind {
            tir::StmtKind::Let(l) => {
                let v = l.init.map(|expr| self.compile_expr(expr)).unwrap_or(self.vals.zero.into());
                let ptr = self.alloca(l.pat.ty, l.pat.id);
                self.def(l.pat.id, ptr);
                self.position_end();
                self.builder.build_store(ptr, v);
            }
            tir::StmtKind::Expr(expr) => {
                self.compile_expr(expr);
            }
        };
    }

    fn curr_fn(&self) -> FunctionValue<'tcx> {
        *self.fns.last().unwrap()
    }

    fn position_end(&mut self) {
        let bb = *self.curr_fn().get_basic_blocks().last().unwrap();
        self.builder.position_at_end(bb);
    }

    fn alloca(&self, ty: Ty, t: impl Display) -> PointerValue<'tcx> {
        let basic_block = self.curr_fn().get_first_basic_block().unwrap();
        match &basic_block.get_first_instruction() {
            Some(inst) => self.builder.position_before(inst),
            None => self.builder.position_at_end(basic_block),
        };
        self.builder.build_alloca(self.llvm_ty(ty), &t.to_string())
    }

    fn compile_block(&mut self, block: &tir::Block) -> BasicValueEnum<'tcx> {
        block.stmts.into_iter().for_each(|stmt| self.compile_stmt(stmt));
        self.compile_expr(block.expr.unwrap())
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
        let cmp = match op {
            ast::BinOp::Lt =>
                self.builder.build_float_compare(FloatPredicate::OLT, l, r, "fcmp_lt"),
            ast::BinOp::Gt =>
                self.builder.build_float_compare(FloatPredicate::OGT, l, r, "fcmp_gt"),
            ast::BinOp::Mul | ast::BinOp::Div | ast::BinOp::Add | ast::BinOp::Sub => unreachable!(),
        };
        self.builder.build_int_z_extend(cmp, self.ctx.i64_type(), "i1->i64")
    }

    fn compile_const(&mut self, c: &Const) -> BasicValueEnum<'tcx> {
        match c.kind {
            ConstKind::Floating(f) => self.ctx.f64_type().const_float(f).into(),
            ConstKind::Integral(i) => self.ctx.i64_type().const_int(i, false).into(),
        }
    }
}
