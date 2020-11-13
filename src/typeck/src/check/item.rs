use ir::DefId;
use lcore::queries::Queries;
use lcore::TyCtx;

pub fn provide(queries: &mut Queries) {
    *queries = Queries { validate_item_type, ..*queries }
}

fn validate_item_type<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) {
    match tcx.defs().get(def_id) {
        ir::DefNode::Item(item) => match item.kind {
            ir::ItemKind::Fn(..) => {}
            ir::ItemKind::Enum(..) | ir::ItemKind::Struct(..) => self::validate_adt(tcx, def_id),
            ir::ItemKind::Use(..) => {}
            ir::ItemKind::Extern(..) => {}
            ir::ItemKind::Impl { .. } => {}
            ir::ItemKind::TypeAlias(..) => {}
            ir::ItemKind::Mod(..) => {}
        },
        ir::DefNode::ImplItem(_) => {}
        ir::DefNode::ForeignItem(_) => {}
        ir::DefNode::Ctor(_) => {}
        ir::DefNode::Field(_) => {}
        ir::DefNode::Variant(_) => {}
        ir::DefNode::TyParam(_) => {}
    }
}

/// we check for things such as the following
/// incorrect number of generic arguments of the fields in the adt decl
/// unrepresentable adts (i.e. infinite sized) (todo)
fn validate_adt<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) {
    let adt = tcx.adt_ty(def_id);

    // we validate the fields of an adt by converting all the fields from
    // `ir::Ty` to `ty::Ty` using `ir_ty_to_ty`
    // (this is implicitly what `type_of` will do to the field)
    // the conversion step performs the necessary checks for correctness
    for variant in &adt.variants {
        for field in variant.fields {
            tcx.type_of(field.def_id);
        }
    }
}
