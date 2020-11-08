#[macro_export]
macro_rules! arena_types {
    ($macro:path, $args:tt, $tcx:lifetime) => (
        $macro!($args, [
            // HIR types
            [few] ir: ir::Ir<$tcx>,
            [] bodies: ir::Body<$tcx>,
            [] arms: ir::Arm<$tcx>,
            [] blocks: ir::Block<$tcx>,
            [] field_decls: ir::FieldDecl<$tcx>,
            [] field_pats: ir::FieldPat<$tcx>,
            [] foreign_items: ir::ForeignItem<$tcx>,
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
            [] locals: ir::Let<$tcx>,
            [] params: ir::Param<$tcx>,
            [] pats: ir::Pattern<$tcx>,
            [] paths: ir::Path<$tcx>,
            [] qpaths: ir::QPath<$tcx>,
            [] path_segments: ir::PathSegment<$tcx>,
            [] stmts: ir::Stmt<$tcx>,
            [] tys: ir::Ty<$tcx>,
            [] variants: ir::Variant<$tcx>,
        ], $tcx);
    )
}
