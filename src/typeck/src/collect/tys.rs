//! collect item types

use index::Idx;
use ir::{CtorKind, DefId, VariantIdx};
use lcore::queries::Queries;
use lcore::ty::{AdtKind, AdtTy, FieldTy, TyCtx, TypeError, VariantTy};
use rustc_hash::FxHashMap;

crate fn provide(queries: &mut Queries) {
    *queries = Queries { adt_ty, ..*queries }
}

fn adt_ty<'tcx>(tcx: TyCtx<'tcx>, def_id: DefId) -> &'tcx AdtTy {
    let item = match tcx.defs().get(def_id) {
        ir::DefNode::Item(item) => item,
        _ => panic!(),
    };

    let (kind, variants) = match item.kind {
        ir::ItemKind::Enum(_, variants) => {
            let variants = variants.iter().map(|variant| self::variant_ty(tcx, variant)).collect();
            (AdtKind::Enum, variants)
        }
        ir::ItemKind::Struct(_, kind) => {
            // little bit hacky, turning the variant kind into a variant...
            let &ir::Item { id, span, ident, .. } = item;
            let variant =
                ir::Variant { id, ident, span, adt_def_id: id.def, kind, idx: VariantIdx::new(0) };
            let variant = std::iter::once(&variant).map(|v| self::variant_ty(tcx, v)).collect();

            (AdtKind::Struct, variant)
        }
        _ => panic!(),
    };

    tcx.mk_adt(def_id, kind, item.ident, variants)
}

fn variant_ty<'tcx>(tcx: TyCtx<'tcx>, variant: &ir::Variant<'tcx>) -> VariantTy {
    let &ir::Variant { id, ident, kind, .. } = variant;

    let mut seen = FxHashMap::default();
    let fields = kind
        .fields()
        .iter()
        .map(|f| {
            if let Some(span) = seen.insert(f.ident, f.span) {
                tcx.sess.emit_error(
                    vec![f.span, span],
                    TypeError::FieldAlreadyDeclared(f.ident, ident),
                );
            }
            FieldTy { def_id: f.id.def, ident: f.ident, vis: f.vis }
        })
        .collect();

    VariantTy { def_id: id.def, ident, fields, ctor_kind: CtorKind::from(&kind) }
}
