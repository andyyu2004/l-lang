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
            [] fields: FieldTy<$tcx>,
            [] mir: Mir<$tcx>,
            [] typeck_tables: TypeckTables<$tcx>,
            [] typarams: TyParam<'tcx>,
            [] types: Type<$tcx>,
        ], $tcx);
    )
}

arena_types!(arena::declare_arena, [], 'tcx);

// collective arena which contains all main arenas
// #[derive(Default)]
// pub struct CoreArenas<'tcx> {
//     drop: DropArena,
//     dropless: DroplessArena,
//     tys: TypedArena<Type<'tcx>>,
//     consts: TypedArena<Const<'tcx>>,
//     // phantom data for each type that may be allocated in drop
//     def_marker: PhantomData<Definitions<'tcx>>,
//     ir_marker: PhantomData<ir::Ir<'tcx>>,
//     // typeck_tables_marker: PhantomData<TypeckTables<'tcx>>,
//     tmp: PhantomData<&'tcx ()>,
// }

// impl<'tcx> CoreArenas<'tcx> {
//     pub fn alloc<T>(&self, t: T) -> &T {
//         if !std::mem::needs_drop::<T>() {
//             self.dropless.alloc(t)
//         } else {
//             unsafe { self.drop.alloc(t) }
//         }
//     }

//     pub fn alloc_iter<I, T>(&self, t: I) -> &[T]
//     where
//         I: IntoIterator<Item = T>,
//     {
//         self.dropless.alloc_from_iter(t)
//     }

//     pub fn alloc_raw(&self, layout: Layout) -> *mut u8 {
//         self.dropless.alloc_raw(layout)
//     }

//     pub fn alloc_ty(&'tcx self, kind: TyKind<'tcx>) -> Ty<'tcx> {
//         let flags = kind.ty_flags();
//         let ty_structure = Type { kind, flags };
//         self.tys.alloc(ty_structure)
//     }

//     pub fn alloc_const(&'tcx self, c: Const<'tcx>) -> &'tcx Const<'tcx> {
//         self.consts.alloc(c)
//     }
// }
