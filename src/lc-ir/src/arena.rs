#[macro_export]
macro_rules! arena_types {
    ($macro:path, $tcx:lifetime) => (
        $macro!([
            // HIR types
            [few] ir: ir::Ir<$tcx>,
            [] bodies: ir::Body<$tcx>,
            [] arms: ir::Arm<$tcx>,
            [] blocks: ir::Block<$tcx>,
            [] generic_args: ir::GenericArgs<$tcx>,
            [] generics: ir::Generics<$tcx>,
            [] expr: ir::Expr<$tcx>,
            [] field: ir::Field<$tcx>,
            [] field_decls: ir::FieldDecl<$tcx>,
            [] field_pats: ir::FieldPat<$tcx>,
            [] fn_sig: ir::FnSig<$tcx>,
            [] foreign_items: ir::ForeignItem<$tcx>,
            [] items: ir::Item<$tcx>,
            [] impl_items: ir::ImplItem<$tcx>,
            [] impl_item_refs: ir::ImplItemRef,
            [] locals: ir::Let<$tcx>,
            [] params: ir::Param<$tcx>,
            [] pats: ir::Pattern<$tcx>,
            [] paths: ir::Path<$tcx>,
            [] qpaths: ir::QPath<$tcx>,
            [] path_segments: ir::PathSegment<$tcx>,
            [] stmts: ir::Stmt<$tcx>,
            [] trait_item: ir::TraitItem<$tcx>,
            [] trait_item_refs: ir::TraitItemRef,
            [] ty_param: ir::TyParam<$tcx>,
            [] tys: ir::Ty<$tcx>,
            [] variants: ir::Variant<$tcx>,
        ]);
    )
}
