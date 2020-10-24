#[macro_export]
macro_rules! arena_types {
    ($macro:path, $args:tt, $tcx:lifetime) => (
        $macro!($args, [
            // HIR types
            [few] ir: ir::Ir<$tcx>,
            [] bodies: ir::Body<$tcx>,
            [] arm: ir::Arm<$tcx>,
            [] block: ir::Block<$tcx>,
            [] field_decls: ir::FieldDecl<$tcx>,
            [] field_pats: ir::FieldPat<$tcx>,
            [] generic_args: ir::GenericArgs<$tcx>,
            [] generics: ir::Generics<$tcx>,
            // [] generic_bound: ir::GenericBound<$tcx>,
            [] ty_param: ir::TyParam<$tcx>,
            [] expr: ir::Expr<$tcx>,
            [] field: ir::Field<$tcx>,
            // [] field_pat: ir::FieldPat<$tcx>,
            [] fn_sig: ir::FnSig<$tcx>,
            // [] foreign_item: ir::ForeignItem<$tcx>,
            [] impl_item_refs: ir::ImplItemRef,
            [] local: ir::Let<$tcx>,
            [] param: ir::Param<$tcx>,
            [] pat: ir::Pattern<$tcx>,
            [] path: ir::Path<$tcx>,
            [] qpath: ir::QPath<$tcx>,
            [] path_segment: ir::PathSegment<$tcx>,
            [] stmt: ir::Stmt<$tcx>,
            [] ty: ir::Ty<$tcx>,
        ], $tcx);
    )
}
