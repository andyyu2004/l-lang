use crate::ast;
use crate::lexer::symbol;
use crate::tir;
use crate::ty::{Const, ConstKind, Ty, TyKind};
use crate::typeck::TyCtx;
use inkwell::types::{BasicType, BasicTypeEnum, FloatType};
use inkwell::values::{BasicValue, BasicValueEnum, FloatValue, FunctionValue, IntValue};
use inkwell::FloatPredicate;
use inkwell::{builder::Builder, context::Context, module::Module, passes::PassManager};
use itertools::Itertools;

crate struct CodegenCtx<'tcx> {
    pub tcx: TyCtx<'tcx>,
    pub ctx: &'tcx Context,
    pub builder: Builder<'tcx>,
    pub fpm: PassManager<FunctionValue<'tcx>>,
    pub module: Module<'tcx>,
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
        Self { tcx, ctx, module, fpm, builder: ctx.create_builder() }
    }

    /// returns the main function
    pub fn compile_tir(&mut self, prog: &tir::Prog) -> FunctionValue<'tcx> {
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
        let (_forall, fn_ty) = ty.expect_scheme();
        let (arg_tys, ret_ty) = fn_ty.expect_fn();
        let llvm_arg_tys = arg_tys.into_iter().map(|ty| self.llvm_ty(ty)).collect_vec();
        let llvm_fn_ty = self.llvm_ty(ret_ty).fn_type(&llvm_arg_tys, false);
        let fn_val = self.module.add_function(&item.ident.to_string(), llvm_fn_ty, None);
        let basic_block = self.ctx.append_basic_block(fn_val, "block");
        self.builder.position_at_end(basic_block);
        // TODO params
        let body = self.compile_expr(body.expr);
        self.builder.build_return(Some(&body));
        assert!(fn_val.verify(true));
        self.fpm.run_on(&fn_val);
        fn_val
    }

    fn llvm_ty(&self, ty: Ty) -> BasicTypeEnum<'tcx> {
        match ty.kind {
            TyKind::Bool => todo!(),
            TyKind::Char => todo!(),
            TyKind::Num => self.ctx.f64_type().into(),
            TyKind::Error => todo!(),
            TyKind::Array(_) => todo!(),
            TyKind::Fn(_, _) => todo!(),
            TyKind::Tuple(_) => todo!(),
            TyKind::Param(_) => todo!(),
            TyKind::Scheme(_, _) => todo!(),
            TyKind::Infer(_) => unreachable!(),
        }
    }

    fn compile_expr(&mut self, expr: &tir::Expr) -> BasicValueEnum<'tcx> {
        match expr.kind {
            tir::ExprKind::Const(c) => self.compile_const(c),
            tir::ExprKind::Bin(op, l, r) => self.compile_bin(op, l, r),
            tir::ExprKind::Unary(_, _) => todo!(),
            tir::ExprKind::Block(block) => self.compile_block(block),
            tir::ExprKind::VarRef(_) => todo!(),
            tir::ExprKind::ItemRef(_) => todo!(),
            tir::ExprKind::Tuple(_) => todo!(),
            tir::ExprKind::Lambda(_) => todo!(),
            tir::ExprKind::Call(_, _) => todo!(),
            tir::ExprKind::Match(_, _) => todo!(),
        }
    }

    fn compile_stmt(&mut self, stmt: &tir::Stmt) {
    }

    fn compile_block(&mut self, block: &tir::Block) -> BasicValueEnum<'tcx> {
        // TODO stmts
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
            ast::BinOp::Lt =>
                self.builder.build_float_compare(FloatPredicate::ULT, lhs, rhs, "tmpfcmp").into(),
            ast::BinOp::Gt =>
                self.builder.build_float_compare(FloatPredicate::ULT, rhs, lhs, "tmpfcmp").into(),
        }
    }

    fn compile_const(&mut self, c: &Const) -> BasicValueEnum<'tcx> {
        match c.kind {
            ConstKind::Floating(f) => self.ctx.f64_type().const_float(f).into(),
            ConstKind::Integral(i) => self.ctx.i64_type().const_int(i, false).into(),
        }
    }
}
