use super::cfg::Cfg;
use lc_ast::Mutability;
use lc_index::IndexVec;
use ir::VariantIdx;
use itertools::Itertools;
use lc_core::mir::*;
use lc_core::ty::{Ty, TyCtx, VariantTy};
use lc_span::Span;

pub fn build_variant_ctor<'tcx>(
    tcx: TyCtx<'tcx>,
    variant: &'tcx ir::Variant<'tcx>,
) -> &'tcx Mir<'tcx> {
    let ty = tcx.type_of(variant.adt_def_id);
    let (adt_ty, _) = ty.expect_adt();
    let idx = variant.idx;
    let variant_ty = &adt_ty.variants[idx];
    build_variant_ctor_inner(tcx, ty, idx, variant_ty).unwrap()
}

/// constructs the mir for a single variant constructor (if it is a function)
fn build_variant_ctor_inner<'tcx>(
    tcx: TyCtx<'tcx>,
    ret_ty: Ty<'tcx>,
    variant_idx: VariantIdx,
    variant: &VariantTy,
) -> Option<&'tcx Mir<'tcx>> {
    // don't construct any mir for a constructor that is not a function
    if !variant.ctor_kind.is_function() {
        return None;
    }

    // TODO get a proper span
    let info = SpanInfo { span: Span::default() };
    let (adt, substs) = ret_ty.expect_adt();

    let mut vars = IndexVec::<VarId, Var<'tcx>>::default();
    let mut alloc_var = |info: SpanInfo, kind: VarKind, ty: Ty<'tcx>| {
        let var = Var { mtbl: Mutability::Imm, info, kind, ty };
        vars.push(var)
    };

    let mut cfg = Cfg::default();
    let lvalue = alloc_var(info, VarKind::Ret, ret_ty).into();

    // the `fields` of the variant are essentially the parameters of the constructor function
    let fields = variant
        .fields
        .iter()
        .map(|param| alloc_var(info, VarKind::Arg, param.ty(tcx, substs)))
        .map(Lvalue::new)
        .map(Operand::Lvalue)
        .collect_vec();

    let rvalue = Rvalue::Adt { adt, variant_idx, substs, fields };
    cfg.push_assignment(info, ENTRY_BLOCK, lvalue, rvalue);
    cfg.terminate(info, ENTRY_BLOCK, TerminatorKind::Return);
    let body = Mir { basic_blocks: cfg.basic_blocks, vars, argc: variant.fields.len() };
    Some(tcx.alloc(body))
}
