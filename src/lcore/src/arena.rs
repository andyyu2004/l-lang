// use crate::ir::{self, Definitions};
// use crate::ty::{Const, List, SubstsRef, Ty, TyFlag, TyKind, TyS};
// use crate::typeck::TypeckTables;
use crate::mir::Mir;
use crate::ty::*;

#[macro_export]
macro_rules! arena_types {
    ($macro:path, $args:tt, $tcx:lifetime) => (
        $macro!($args, [
            [] adt_def: AdtTy<$tcx>,
            [] consts: Const<$tcx>,
            [] fields: FieldTy,
            [] generics: Generics<$tcx>,
            [] inherent_impls: InherentImpls,
            [] mir: Mir<$tcx>,
            [] typeck_tables: TypeckTables<$tcx>,
            [] typarams: TyParam<'tcx>,
            [] types: Type<$tcx>,
        ], $tcx);
    )
}

arena_types!(arena::declare_arena, [], 'tcx);
