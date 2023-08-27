use crate::mir::Mir;
use crate::ty::*;

#[macro_export]
macro_rules! arena_types {
    ($macro:path, $tcx:lifetime) => (
        $macro!([
            [] adt_def: AdtTy,
            [] consts: Const<$tcx>,
            [] fields: FieldTy,
            [] generics: Generics<$tcx>,
            [] inherent_impls: InherentImpls,
            [] instances: Instances<$tcx>,
            [] mir: Mir<$tcx>,
            [] trait_impls: TraitImpls,
            [] typeck_tables: TypeckTables<$tcx>,
            [] typarams: TyParam<'tcx>,
            [] types: Type<$tcx>,
        ]);
    )
}

arena_types!(lc_arena::declare_arena, 'tcx);
